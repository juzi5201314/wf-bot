use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent,
    MessageSendToSourceTrait, Module,
};

use crate::wf_api::{cetus_cycle, gen_cetus_info};

#[event]
async fn cmd(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content == "三傻" || content =="3傻" || content == "夜灵" {
        match cetus_cycle().await {
            Ok(data) => {
                event.send_message_to_source(gen_cetus_info(&data)).await?;
            }
            Err(err) => {
                tracing::warn!("eidolon error: {}", err);
                event
                    .send_message_to_source(
                        "希图斯状态接口出现了错误, 等等再试吧".parse_message_chain(),
                    )
                    .await?;
            }
        };
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn module() -> Module {
    module!("eidolon", "三傻", cmd)
}
