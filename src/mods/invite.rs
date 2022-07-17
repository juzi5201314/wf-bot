use proc_qq::{event, module, JoinGroupRequestEvent, Module};

#[event]
async fn invite_to_group(event: &JoinGroupRequestEvent) -> anyhow::Result<bool> {
    tracing::debug!("{:?}", &event.inner);
    if let Some(uin) = event.inner.invitor_uin {
        if uin == dotenv::var("owner")?.parse::<i64>()? {
            event.accept().await?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn module() -> Module {
    module!("invite", "邀请", invite_to_group)
}
