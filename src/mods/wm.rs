use std::sync::Arc;

use compact_str::CompactStr;
use itertools::Itertools;
use once_cell::sync::Lazy;
use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent,
    MessageSendToSourceTrait, Module,
};

use crate::wf_api::{
    update_items_db, update_rivens_db, wm_item, wm_riven, OrderType, UserStatus, RIVEN_ATTR,
};

static ITEMS_DB: Lazy<Arc<sled::Db>> =
    Lazy::new(|| Arc::new(sled::open("items_db").expect("ITEMS_DB open err")));
static RIVENS_DB: Lazy<Arc<sled::Db>> =
    Lazy::new(|| Arc::new(sled::open("rivens_db").expect("RIVENS_DB open err")));

#[event]
async fn cmd(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content.starts_with("wm") {
        let mut param = content.trim_start_matches("wm").trim();
        let mod_lvl = if param.starts_with("+") {
            if let Some((lvl, other)) = param.trim_start_matches("+").split_once(" ") {
                if let Ok(lvl) = lvl.parse::<i32>() {
                    param = other;
                    Some(lvl)
                } else {
                    event
                        .send_message_to_source("mod等级必须是数字".parse_message_chain())
                        .await?;
                    return Ok(true);
                }
            } else {
                None
            }
        } else {
            None
        };
        let item_name = param.to_ascii_lowercase().replace(" ", "");
        match ITEMS_DB.get(item_name.as_bytes())? {
            None => {
                event
                    .send_message_to_source(
                        format!("找不到在售物品 {}", item_name).parse_message_chain(),
                    )
                    .await?;
            }
            Some(url_name) => {
                let url_name = String::from_utf8_lossy(url_name.as_ref());
                let orders = wm_item(url_name.as_ref()).await?;

                let mut orders_info = String::with_capacity(512);

                orders
                    .into_iter()
                    .filter(|order| matches!(order.user.status, UserStatus::InGame))
                    .filter(|order| matches!(order.order_type, OrderType::Sell))
                    .filter(|order| order.region == "en")
                    .filter(|order| order.visible)
                    .filter(|order| {
                        if let (Some(lvl), Some(lvl2)) = (mod_lvl, order.mod_rank) {
                            lvl == lvl2
                        } else {
                            true
                        }
                    })
                    .sorted_unstable_by(|l, r| l.platinum.cmp(&r.platinum))
                    .take(4)
                    .for_each(|order| {
                        orders_info.push_str(
                            format!(
                                "{name} 卖 ${platinum}, 库存 {count} 个",
                                name = &order.user.ingame_name,
                                platinum = order.platinum,
                                count = order.quantity,
                            )
                            .as_str(),
                        );
                        if let Some(rank) = order.mod_rank {
                            orders_info.push_str(format!(" ({} 级)", rank).as_str());
                        }
                        orders_info.push('\n');
                    });

                event
                    .send_message_to_source(
                        format!("{}~ 截至游戏中卖家价格最低前4条", orders_info).parse_message_chain(),
                    )
                    .await?;
            }
        }
        Ok(true)
    } else if content.starts_with("zk") {
        let mut positive_stats = Vec::new();
        let mut negative_stats = CompactStr::new_inline("");

        let mut params = content.split_whitespace().peekable();
        params.next().unwrap();

        loop {
            if let Some(param) = params.peek() {
                let mut chs = param.chars().peekable();
                if let Some('+') = chs.peek() {
                    chs.next().unwrap();
                    let attr = CompactStr::from_iter(chs);
                    if let Some(url_name) = RIVEN_ATTR.get(&attr) {
                        positive_stats.push(CompactStr::new(url_name));
                    } else {
                        event
                            .send_message_to_source(
                                format!("找不到词条: {}", attr).parse_message_chain(),
                            )
                            .await?;
                    }
                } else if let Some('-') = chs.peek() {
                    chs.next().unwrap();
                    let attr = CompactStr::from_iter(chs);
                    if let Some(url_name) = RIVEN_ATTR.get(&attr) {
                        negative_stats = CompactStr::new(url_name);
                    } else {
                        event
                            .send_message_to_source(
                                format!("找不到词条: {}", attr).parse_message_chain(),
                            )
                            .await?;
                    }
                } else {
                    break;
                }
                params.next().unwrap();
            } else {
                break;
            }
        }

        let item_name = params.join("");
        match RIVENS_DB.get(&item_name)? {
            None => {
                event
                    .send_message_to_source(
                        format!("找不到在售的 {} 紫卡", item_name).parse_message_chain(),
                    )
                    .await?;
            }
            Some(url_name) => {
                let mut auctions_info = String::with_capacity(1024);
                let auctions = wm_riven(
                    String::from_utf8_lossy(url_name.as_ref()).as_ref(),
                    &positive_stats.join(","),
                    &negative_stats,
                )
                .await?;

                auctions
                    .into_iter()
                    .filter(|auction| matches!(auction.owner.status, UserStatus::InGame))
                    .filter(|auction| !auction.private)
                    .filter(|auction| auction.visible)
                    .filter(|auction| !auction.closed)
                    .sorted_unstable_by(|l, r| {
                        l.buyout_price
                            .unwrap_or(l.starting_price)
                            .cmp(&r.buyout_price.unwrap_or(r.starting_price))
                    })
                    .take(3)
                    .for_each(|auction| {
                        auctions_info.push_str(&format!(
                            "{} {} {}段 {}洗 {}级 {}槽 ${}",
                            &item_name,
                            &auction.item.name,
                            auction.item.mastery_level,
                            auction.item.re_rolls,
                            auction.item.mod_rank,
                            auction.item.polarity.nickname(),
                            auction.buyout_price.unwrap_or(auction.starting_price),
                        ));

                        auction.item.attributes.iter().for_each(|attr| {
                            let attr_name = RIVEN_ATTR
                                .entries()
                                .find(|(_, url_name2)| attr.url_name == url_name2)
                                .map(|(attr_name, _)| *attr_name)
                                .unwrap_or(&attr.url_name);
                            auctions_info.push_str(&format!(
                                "\n  {}{} {}",
                                if attr.value >= 0f64 { "+" } else { "" },
                                attr.value,
                                attr_name
                            ))
                        });

                        auctions_info.push('\n');
                    });
                event
                    .send_message_to_source(
                        format!("{}~ 截至游戏中卖家价格最低前3条", auctions_info)
                            .parse_message_chain(),
                    )
                    .await?;
            }
        }
        Ok(true)
    } else if content == "update_items_db" {
        let num = update_items_db(ITEMS_DB.clone()).await?;
        event
            .send_message_to_source(
                format!(
                    "成功储存 {} 条数据, 数据库中共有 {} 条数据",
                    num,
                    ITEMS_DB.len()
                )
                .parse_message_chain(),
            )
            .await?;
        Ok(true)
    } else if content == "update_rivens_db" {
        let num = update_rivens_db(RIVENS_DB.clone()).await?;
        event
            .send_message_to_source(
                format!(
                    "成功储存 {} 条数据, 数据库中共有 {} 条数据",
                    num,
                    RIVENS_DB.len()
                )
                .parse_message_chain(),
            )
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn module() -> Module {
    module!("arbitration", "仲裁", cmd)
}
