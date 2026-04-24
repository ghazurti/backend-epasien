use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct ServiceAccountClaims {
    iss: String,
    scope: String,
    aud: String,
    exp: u64,
    iat: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub struct FcmClient {
    http: Client,
    project_id: String,
    client_email: String,
    private_key: String,
    cached_token: Option<(String, u64)>,
}

impl FcmClient {
    pub fn new(project_id: String, client_email: String, private_key: String) -> Self {
        Self {
            http: Client::new(),
            project_id,
            client_email,
            private_key,
            cached_token: None,
        }
    }

    async fn get_access_token(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp() as u64;

        if let Some((token, expiry)) = &self.cached_token {
            if *expiry > now + 300 {
                return Ok(token.clone());
            }
        }

        let claims = ServiceAccountClaims {
            iss: self.client_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            exp: now + 3600,
            iat: now,
        };

        let header = Header::new(Algorithm::RS256);
        let key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())?;
        let jwt = encode(&header, &claims, &key)?;

        let resp = self
            .http
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        self.cached_token = Some((resp.access_token.clone(), now + 3600));
        Ok(resp.access_token)
    }

    pub async fn send(
        &mut self,
        token: &str,
        title: &str,
        body: &str,
        data: Option<serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;

        let mut message = serde_json::json!({
            "message": {
                "token": token,
                "notification": {
                    "title": title,
                    "body": body
                },
                "android": {
                    "priority": "high",
                    "notification": {
                        "sound": "default",
                        "channel_id": "epasien_channel"
                    }
                }
            }
        });

        if let Some(d) = data {
            message["message"]["data"] = d;
        }

        let response = self
            .http
            .post(format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                self.project_id
            ))
            .bearer_auth(&access_token)
            .json(&message)
            .send()
            .await?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(format!("FCM error: {}", err_text).into());
        }

        Ok(())
    }
}

pub type SharedFcmClient = Arc<Mutex<FcmClient>>;

pub fn create_fcm_client() -> Option<SharedFcmClient> {
    let project_id = std::env::var("FCM_PROJECT_ID").ok()?;
    let client_email = std::env::var("FCM_CLIENT_EMAIL").ok()?;
    let private_key = std::env::var("FCM_PRIVATE_KEY")
        .ok()?
        .replace("\\n", "\n");

    Some(Arc::new(Mutex::new(FcmClient::new(
        project_id,
        client_email,
        private_key,
    ))))
}
