use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent,
    MessageSendToSourceTrait, Module,
};

use crate::wf_api::{arbitration, gen_arbitration_info};

#[event]
async fn cmd(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content == "仲裁" {
        if rand::random::<u8>() % 64 == 0 {
            event
                .send_message_to_source(format!(
                    "节点: {node} \n剩余时间(约): {time} 分钟 \n类型: {ty} \n敌人: {enemy} \n个人评价: {level}",
                    node = "FuckMother (地球)",
                    time = 45,
                    ty = "刺杀",
                    enemy = "de的妈",
                    level = "好图",
                ).parse_message_chain()).await?;
        } else {
            match arbitration().await {
                Ok(data) => {
                    event
                        .send_message_to_source(gen_arbitration_info(&data))
                        .await?;
                }
                Err(err) => {
                    tracing::warn!("arbitration error: {}", err);
                    event
                        .send_message_to_source(format!(
                            "节点: {node} \n剩余时间(约): {time} 分钟 \n类型: {ty} \n敌人: {enemy} \n个人评价: {level}",
                            node = "SaveMother (地府)",
                            time = -1,
                            ty = "救援",
                            enemy = "de的妈",
                            level = "好图, 但你不能救一个不存在的生物",
                        ).parse_message_chain()).await?;
                }
            };
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn module() -> Module {
    module!("arbitration", "仲裁", cmd)
}
