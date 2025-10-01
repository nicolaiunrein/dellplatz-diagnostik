use crate::types::*;
use color_eyre::Result;
use reqwest::multipart;
use reqwest::Client;

const API_ENDPOINT: &str = "http://localhost:30000/forms/chromium/convert/html";

#[tracing::instrument(err)]
pub(crate) async fn generate_pdf(user_id: String, res: &Vec<TestResultRecord>) -> Result<()> {
    let html = format_html(user_id, res);

    let client = Client::new();

    let part = multipart::Part::text(html.to_string())
        .file_name("index.html")
        .mime_str("text/html")?;

    let form = multipart::Form::new().part("files", part);

    let resp = client.post(API_ENDPOINT).multipart(form).send().await?;

    if !resp.status().is_success() {
        let err = resp.text().await?;
        color_eyre::eyre::bail!("Failed to generate PDF: {err}");
    }

    let bytes = resp.bytes().await?;
    tokio::fs::write("output.pdf", &bytes).await?;

    tracing::info!("PDF written to output.pdf");

    Ok(())
}

const STYLE: &str = include_str!("../assets/report.css");

fn format_row(row: &TestResultRecord) -> String {
    let TestResultRecord {
        question_txt,
        answer_txt,
        answer_value,
        ..
    } = row;
    format!(
        r"
<tr>
<td>{question_txt}</td>
<td>{answer_txt}</td>
<td>{answer_value}</td>
</tr>
    "
    )
}

fn format_html(user_id: String, rows: &Vec<TestResultRecord>) -> String {
    let sum = rows.into_iter().map(|r| r.answer_value).sum::<usize>();
    let rows = rows.into_iter().map(format_row).collect::<String>();
    format!(
        r#"
<!doctype html>
<html>
  <head>
    <style>
    {STYLE}
    </style>
  </head>
  <body>
    <h2>Test Report AQ</h2>
    <div class="patienten-id">
    <b>Patienten ID:</b>
    <pre>{user_id}</pre>
    </div>

    <table>
      <tr>
        <th>Frage</th>
        <th>Antwort</th>
        <th>Score</th>
      </tr>
      {rows}
      <tfoot>
      <tr>
      <td colspan=2>SUMME</td>
      <td>{sum}</td>
      </tr>
      </tfoot>
    </table>
  </body>
</html>
"#
    )
}
