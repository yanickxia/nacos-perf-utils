use log::{info, warn};
use std::error::Error;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

pub(crate) struct Runtime {
    sched: JobScheduler,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            sched: JobScheduler::new().unwrap(),
        }
    }
}

impl Runtime {
    pub async fn run(&mut self, nacos: String, from: u32, num: u32) -> Result<(), Box<dyn Error>> {
        let mut num = num;

        while num > 0 {
            let port = from + num;
            let nacos_addr = nacos.clone();
            let nacos = Nacos::new(nacos_addr.clone().to_string(), port);
            nacos.new_instance().await?;
            let jja = Job::new_repeated_async(Duration::from_secs(5), move |_uuid, _l| {
                let nacos = nacos.clone();
                Box::pin(async move {
                    nacos.heart_beat().await.unwrap();
                })
            })
            .unwrap();

            self.sched.add(jja)?;

            num -= 1
        }

        match self.sched.start() {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
#[derive(Clone)]
pub(crate) struct Nacos {
    nacos: String,
    port: u32,
    client: reqwest::Client,
}

impl Nacos {
    pub fn new(nacos: String, port: u32) -> Self {
        Nacos {
            nacos,
            port,
            client: reqwest::Client::new(),
        }
    }

    pub async fn new_instance(&self) -> Result<(), Box<dyn Error>> {
        let resp = self
            .client
            .post(self.nacos.clone() + "/nacos/v1/ns/instance")
            .query(&[
                ("port", self.port.to_string().as_str()),
                ("ip", "localhost"),
                ("ephemeral", "true"),
                ("serviceName", format!("mock-{}", self.port).as_str()),
            ])
            .send()
            .await?;

        if resp.status().is_success() {
            info!(
                "new instance success service: {}",
                format!("mock-{}", self.port).as_str()
            );
        } else {
            warn!("new instance failed");
        }
        Ok(())
    }

    pub async fn heart_beat(&self) -> Result<(), Box<dyn Error>> {
        let beat = format!(
            r#"{{"port":{},"ip":"localhost","serviceName":"mock-{}"}}"#,
            self.port, self.port
        );

        let resp = self
            .client
            .put(self.nacos.clone() + "/nacos/v1/ns/instance/beat")
            .query(&[
                ("serviceName", format!("mock-{}", self.port).as_str()),
                ("beat", beat.as_str()),
            ])
            .send()
            .await?;

        if resp.status().is_success() {
            info!("heart beat success");
        } else {
            warn!("heart beat failed");
        }

        Ok(())
    }
}
