use std::sync::Arc;
use std::time::Duration;

use crate::wf_api::ArbitrationLevel;
use compact_str::CompactStr;
use proc_qq::re_exports::ricq::Client;
use proc_qq::MessageChainParseTrait;
use time::OffsetDateTime;

pub fn eidolon(client: Arc<Client>) {
    tokio::spawn(async move {
        let mut last_id = CompactStr::new("");
        let mut timer = tokio::time::interval(Duration::from_secs(30));
        loop {
            timer.tick().await;

            match crate::wf_api::cetus_cycle().await {
                Ok(data) => {
                    if data.id == last_id {
                        continue
                    } else {
                        // secs
                        let remaining = (data.expiry - OffsetDateTime::now_utc()).whole_seconds();
                        if remaining < 700 && data.is_day {
                            let targets = dotenv::var("eidolon_notice")
                                .unwrap_or_default()
                                .split(",")
                                .map(|x| x.parse().unwrap())
                                .collect::<Vec<i64>>();

                            for target in targets {
                                client
                                    .send_group_message(
                                        target,
                                        "3傻还有10分钟. 有人带我吗, 我打碎片位插碎片贼快"
                                            .parse_message_chain(),
                                    )
                                    .await
                                    .unwrap();
                            }

                            last_id = data.id;
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("eidolon timing error: {}", err);
                    continue;
                }
            }
        }
    });
}

pub fn arbitration(client: Arc<Client>) {
    tokio::spawn(async move {
        let mut last_id = CompactStr::new("");

        let mut timer = tokio::time::interval(Duration::from_secs(30));
        loop {
            timer.tick().await;

            let data = match crate::wf_api::arbitration().await {
                Ok(a) => a,
                Err(err) => {
                    tracing::error!("arbitration timing error: {}", err);
                    continue;
                }
            };
            if data.id == last_id {
                continue;
            } else if last_id.is_empty() {
                last_id = data.id;
            } else if let ArbitrationLevel::T0 = ArbitrationLevel::from_data(&data) {
                let targets = dotenv::var("arbitration_notice")
                    .unwrap_or_default()
                    .split(",")
                    .map(|x| x.parse().unwrap())
                    .collect::<Vec<i64>>();

                for target in targets {
                    client
                        .send_group_message(target, "好图!".parse_message_chain())
                        .await
                        .unwrap();
                    client
                        .send_group_message(target, crate::wf_api::gen_arbitration_info(&data))
                        .await
                        .unwrap();
                }

                last_id = data.id
            }
        }
    });
}
