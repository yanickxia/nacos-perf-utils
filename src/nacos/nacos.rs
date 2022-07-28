use std::collections::HashMap;
use log::{info, warn};
use std::error::Error;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use serde::Deserialize;

#[derive(Clone)]
pub struct Config {
    pub nacos: String,
    pub port: u32,
    pub num: u32,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    access_token: String,
}


pub(crate) struct Runtime {
    config: Config,
    sched: JobScheduler,
    client: reqwest::Client,
}


impl Runtime {
    pub fn new(config: Config) -> Self {
        Self {
            sched: JobScheduler::new().unwrap(),
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn run(&mut self, config: Config) -> Result<(), Box<dyn Error>> {
        let mut num = config.num;

        let mut login_info = LoginInfo {
            access_token: "".to_string()
        };
        if self.config.username.is_some() {
            login_info = self.login().await?;
        }
        
        while num > 0 {
            let port = config.port + num;

            let mut nacos_config = config.clone();
            nacos_config.port = port;

            let nacos = Nacos::new(nacos_config, login_info.access_token.to_owned());
            nacos.new_instance().await?;
            let jja = Job::new_repeated_async(Duration::from_secs(5), move |_uuid, _l| {
                let nacos = nacos.clone();
                Box::pin(async move {
                    nacos.heart_beat().await.unwrap();
                })
            }).unwrap();

            self.sched.add(jja)?;

            num -= 1
        }

        match self.sched.start() {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn login(&self) -> Result<LoginInfo, Box<dyn Error>> {
        let username = self.config.username.as_ref().unwrap();
        let password = self.config.password.as_ref().unwrap();

        let mut params = HashMap::new();
        params.insert("username", username.clone());
        params.insert("password", password.clone());

        let resp = self
            .client
            .post(self.config.nacos.clone() + "/nacos/v1/auth/login")
            .form(&params)
            .send()
            .await?
            .json::<LoginInfo>()
            .await?;

        Ok(resp)
    }
}

#[derive(Clone)]
pub(crate) struct Nacos {
    token: String,
    config: Config,
    client: reqwest::Client,
}

impl Nacos {
    pub fn new(config: Config, token: String) -> Self {
        Nacos {
            token,
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn new_instance(&self) -> Result<(), Box<dyn Error>> {
        let resp = self
            .client
            .post(self.config.nacos.clone() + "/nacos/v1/ns/instance")
            .query(&[
                ("port", self.config.port.to_string().as_str()),
                ("ip", "localhost"),
                ("accessToken", self.token.as_str()),
                ("ephemeral", "true"),
                ("serviceName", format!("mock-{}", self.config.port).as_str()),
            ])
            .send()
            .await?;

        if resp.status().is_success() {
            info!(
                "new instance success service: {}",
                format!("mock-{}", self.config.port).as_str()
            );
        } else {
            warn!("new instance failed");
        }
        Ok(())
    }

    pub async fn heart_beat(&self) -> Result<(), Box<dyn Error>> {
        let beat = format!(
            r#"{{"port":{},"ip":"localhost","serviceName":"mock-{}"}}"#,
            self.config.port, self.config.port
        );

        let resp = self
            .client
            .put(self.config.nacos.clone() + "/nacos/v1/ns/instance/beat")
            .query(&[
                ("serviceName", format!("mock-{}", self.config.port).as_str()),
                ("beat", beat.as_str()),
                ("accessToken", self.token.as_str()),
            ])
            .send()
            .await?;

        if resp.status().is_success() {
            info!("instance {}:{} heart beat success", self.config.nacos, self.config.port);
        } else {
            warn!("instance {}:{} heart beat failed", self.config.nacos, self.config.port);
        }

        Ok(())
    }
}
