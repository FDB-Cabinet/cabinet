use derive_builder::Builder;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Builder)]
#[builder(setter(into))]
pub struct Gitlab {
    endpoint: String,
    token: String,
    project_id: u64,
}

impl Gitlab {
    pub async fn create_issue(&self, logs: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let form = reqwest::multipart::Form::new();
        let form = form.file("file", logs).await?;

        let a = client
            .post(format!(
                "https://{}/api/v4/projects/{}/uploads",
                self.endpoint, self.project_id
            ))
            .multipart(form)
            .header("PRIVATE-TOKEN", &self.token)
            .build()?;

        let response = client.execute(a).await?;
        let b = response.text().await?;
        let c = serde_json::from_str::<UploadResponse>(&b)?;

        let params = HashMap::from([
            ("title", "Test Issue".to_string()),
            (
                "description",
                format!(r#"This is the [output]({}) of the test run."#, c.url),
            ),
        ]);

        let a = client
            .post(format!(
                "https://{}/api/v4/projects/{}/issues",
                self.endpoint, self.project_id
            ))
            .query(&params)
            .header("PRIVATE-TOKEN", &self.token)
            .build()?;

        client.execute(a).await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct UploadResponse {
    url: String,
}
