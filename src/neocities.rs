use base64::prelude::*;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct KeyResponse {
    // result: String,
    error_type: Option<String>,
    message: Option<String>,
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GenericResponse {
    // result: String,
    error_type: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListResponse {
    // result: String,
    error_type: Option<String>,
    message: Option<String>,
    files: Option<Vec<Entry>>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    // pub is_directory: bool,
    pub path: String,
    // pub updated_at: String,
    // pub size: Option<u64>,
    pub sha1_hash: Option<String>,
}

#[derive(Debug)]
pub enum UploadError {
    InvalidFileType,
    InvalidAuth,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for UploadError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

#[derive(Debug)]
pub enum DeleteError {
    MissingFiles,
    InvalidAuth,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for DeleteError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

#[derive(Debug)]
pub enum ListError {
    InvalidAuth,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for ListError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

#[derive(Debug, Default)]
pub struct Neocities {
    client: Client,
    pub api_key: Option<String>,
}

impl Neocities {
    pub fn new() -> Self {
        Self { client: Client::new(), api_key: None }
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<bool, reqwest::Error> {
        let response = self
            .client
            .get("https://neocities.org/api/key")
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    BASE64_STANDARD.encode(format!("{}:{}", username, password))
                ),
            )
            .send()
            .await?
            .json::<KeyResponse>()
            .await?;
        match response.error_type.as_deref() {
            Some("invalid_auth") => {
                self.api_key = None;
                Ok(false)
            }
            Some(error) => {
                unimplemented!("(login) {} : {}", error, response.message.unwrap())
            }
            None => {
                self.api_key = response.api_key;
                Ok(true)
            }
        }
    }

    pub async fn upload<T>(&self, files: T) -> Result<(), UploadError>
    where T: IntoIterator<Item = (String, Vec<u8>)> {
        let mut length = 0;
        let mut form = reqwest::multipart::Form::new();
        for (name, file) in files {
            form = form.part(
                name.clone(),
                reqwest::multipart::Part::bytes(file).file_name(name),
            );
            length += 1;
        }
        if length == 0 {
            return Ok(());
        }
        let response = self
            .client
            .post("https://neocities.org/api/upload")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .multipart(form)
            .send()
            .await?
            .json::<GenericResponse>()
            .await?;
        match response.error_type.as_deref() {
            Some("invalid_auth") => Err(UploadError::InvalidAuth),
            Some("invalid_file_type") => Err(UploadError::InvalidFileType),
            Some(error) => {
                unimplemented!("(upload) {} : {}", error, response.message.unwrap())
            }
            _ => Ok(()),
        }
    }

    pub async fn delete<T>(&self, files: T) -> Result<(), DeleteError>
    where T: IntoIterator<Item = String> {
        let mut length = 0;
        let mut form = reqwest::multipart::Form::new();
        for name in files {
            if name == "index.html" {
                continue;
            }
            form = form.text("filenames[]", name);
            length += 1;
        }
        if length == 0 {
            return Ok(());
        }
        let response = self
            .client
            .post("https://neocities.org/api/delete")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .multipart(form)
            .send()
            .await?
            .json::<GenericResponse>()
            .await?;
        match response.error_type.as_deref() {
            Some("missing_files") => Err(DeleteError::MissingFiles),
            Some("invalid_auth") => Err(DeleteError::InvalidAuth),
            Some(error) => {
                unimplemented!("(delete) {} : {}", error, response.message.unwrap())
            }
            _ => Ok(()),
        }
    }

    pub async fn list(&self) -> Result<Vec<Entry>, ListError> {
        let response = self
            .client
            .get("https://neocities.org/api/list")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .send()
            .await?
            .json::<ListResponse>()
            .await?;
        match response.error_type.as_deref() {
            Some("invalid_auth") => Err(ListError::InvalidAuth),
            Some(error) => {
                unimplemented!("(list) {} : {}", error, response.message.unwrap())
            }
            _ => Ok(response.files.unwrap()),
        }
    }
}
