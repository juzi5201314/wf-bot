use std::sync::Arc;
use std::time::Duration;

use compact_str::CompactStr;
use phf::phf_map;
use proc_qq::MessageChainParseTrait;
use proc_qq::re_exports::ricq::msg::MessageChain;
use serde::Deserialize;
use time::OffsetDateTime;

macro_rules! api_url {
    () => {
        "https://api.warframestat.us/pc"
    };
}

macro_rules! wm_api_url {
    () => {
        "https://api.warframe.market/v1"
    };
}

pub static RIVEN_ATTR: phf::Map<&'static str, &'static str> = phf_map! {
    "弹药上限" => "ammo_maximum",
    "c伤" => "damage_vs_corpus",
    "g伤" => "damage_vs_grineer",
    "i伤" => "damage_vs_infested",
    "冰" => "cold_damage",
    "初始连击" => "channeling_damage",
    "重击效率" => "channeling_efficiency",
    "连击时间" => "combo_duration",
    "暴率" => "critical_chance",
    "滑行暴率" => "critical_chance_on_slide_attack",
    "暴伤" => "critical_damage",
    "基伤" => "base_damage_/_melee_damage",
    "电" => "electric_damage",
    "火" => "heat_damage",
    "处决" => "finisher_damage",
    "攻速" => "fire_rate_/_attack_speed",
    "射速" => "fire_rate_/_attack_speed",
    "投射物" => "projectile_speed",
    "冲击" => "impact_damage",
    "弹匣" => "magazine_capacity",
    "多重" => "multishot",
    "毒" => "toxin_damage",
    "穿透" => "punch_through",
    "穿刺" => "puncture_damage",
    "装填" => "reload_speed",
    "范围" => "range",
    "切割" => "slash_damage",
    "触发几率" => "status_chance",
    "触发时间" => "status_duration",
    "后坐力" => "recoil",
    "变焦" => "zoom",
    "额外连击" => "chance_to_gain_extra_combo_count",
    "连击几率" => "chance_to_gain_combo_count",
    "无负" => "none",
    "负" => "has",
};

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Enemy {
    Orokin,
    Corrupted,
    Infested,
    Corpus,
    Grineer,
    Tenno,
}

impl Enemy {
    pub fn nickname(&self) -> &'static str {
        match self {
            Enemy::Orokin => "o佬",
            Enemy::Corrupted => "堕落者",
            Enemy::Infested => "i佬",
            Enemy::Corpus => "c佬",
            Enemy::Grineer => "g佬",
            Enemy::Tenno => "天..天诺?",
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ArbitrationLevel {
    T0,
    T1,
    Bad,
}

impl ArbitrationLevel {
    pub fn nickname(&self) -> &'static str {
        match self {
            ArbitrationLevel::T0 => "打它丫的",
            ArbitrationLevel::T1 => "可以打但没必要",
            ArbitrationLevel::Bad => "垃圾图/未定级",
        }
    }

    pub fn from_data(data: &Arbitration) -> Self {
        if data.node.contains("穀神星") {
            if data.type_key == "Defense" || data.type_key == "Interception" {
                return ArbitrationLevel::T0;
            }
        }

        if data.node.contains("賽德娜") {
            if data.type_key == "Defense" {
                return ArbitrationLevel::T0;
            }
        }

        if data.node.contains("水星") {
            if data.type_key == "Interception" {
                return ArbitrationLevel::T0;
            }
            if data.type_key == "Defense" {
                return ArbitrationLevel::T1;
            }
        }

        if data.node.contains("冥王星") {
            if data.type_key == "Defense" {
                if let Enemy::Corpus = data.enemy {
                    return ArbitrationLevel::T1;
                } else {
                    return ArbitrationLevel::T0;
                }
            }

            if data.type_key == "Dark Sector Defense" {
                return ArbitrationLevel::T0;
            }
        }

        if data.node.contains("地球") {
            if data.type_key == "Defense" || data.type_key == "Interception" {
                return ArbitrationLevel::T1;
            }
        }

        if data.node.contains("海王星") {
            if data.type_key == "Defense" || data.type_key == "Interception" {
                return ArbitrationLevel::T1;
            }
        }

        if data.node.contains("土星") {
            if data.type_key == "Defense" {
                return ArbitrationLevel::T0;
            }
            if data.type_key == "Interception" {
                return ArbitrationLevel::T1;
            }
        }

        if data.node.contains("金星") {
            if data.type_key == "Defense" {
                return ArbitrationLevel::T1;
            }
        }

        if data.node.contains("虛空") || data.node.contains("虚空") {
            if data.type_key == "Interception" {
                return ArbitrationLevel::T1;
            }
        }

        ArbitrationLevel::Bad
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Arbitration {
    pub id: CompactStr,
    #[serde(with = "time::serde::iso8601")]
    pub activation: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub expiry: OffsetDateTime,

    #[serde(rename = "startString")]
    pub start_string: Option<CompactStr>,

    pub active: Option<bool>,
    pub node: CompactStr,
    #[serde(rename = "nodeKey")]
    pub node_key: Option<CompactStr>,

    pub enemy: Enemy,
    #[serde(rename = "enemyKey")]
    pub enemy_key: Option<Enemy>,

    #[serde(rename = "type")]
    pub r#type: CompactStr,
    #[serde(rename = "typeKey")]
    pub type_key: CompactStr,

    pub archwing: bool,
    pub sharkwing: bool,
}

pub async fn arbitration() -> anyhow::Result<Arbitration> {
    reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(concat!(api_url!(), "/arbitration?language=zh"))
        .send()
        .await?
        .json()
        .await
        .map_err(Into::into)
}

pub fn gen_arbitration_info(data: &Arbitration) -> MessageChain {
    // minutes
    let remaining = (data.expiry - OffsetDateTime::now_utc()).whole_minutes();
    format!(
        "节点: {node} \n剩余时间(约): {time} 分钟 \n类型: {ty} \n敌人: {enemy} \n个人评价: {level}",
        node = &data.node,
        time = remaining,
        ty = data.r#type,
        enemy = data.enemy.nickname(),
        level = ArbitrationLevel::from_data(data).nickname(),
    )
    .parse_message_chain()
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum CetusState {
    #[serde(rename = "night")]
    Night,
    #[serde(rename = "day")]
    Day,
}

impl CetusState {
    fn chinese(&self) -> &'static str {
        match self {
            CetusState::Night => "黑夜",
            CetusState::Day => "白天",
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CetusCycle {
    pub id: CompactStr,

    #[serde(with = "time::serde::iso8601")]
    pub expiry: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub activation: OffsetDateTime,

    #[serde(rename = "isDay")]
    pub is_day: bool,

    pub state: CetusState,
}

pub async fn cetus_cycle() -> anyhow::Result<CetusCycle> {
    reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(concat!(api_url!(), "/cetusCycle"))
        .send()
        .await?
        .json()
        .await
        .map_err(Into::into)
}

pub fn gen_cetus_info(data: &CetusCycle) -> MessageChain {
    // minutes
    let remaining = (data.expiry - OffsetDateTime::now_utc()).whole_minutes();
    format!(
        "目前状态: {state} \n剩余时间(约): {time} 分钟",
        state = data.state.chinese(),
        time = remaining,
    )
    .parse_message_chain()
}

pub async fn update_items_db(db: Arc<sled::Db>) -> anyhow::Result<u32> {
    #[derive(Deserialize, Debug, Clone)]
    struct Body {
        payload: Payload,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Payload {
        items: Vec<Item>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Item {
        url_name: CompactStr,
        item_name: CompactStr,
    }

    let payload = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(concat!(wm_api_url!(), "/items"))
        .header("Language", "zh-hans")
        .send()
        .await?
        .json::<Body>()
        .await?
        .payload;

    let mut num = 0u32;
    payload.items.into_iter().try_for_each(|item| {
        db.insert(
            item.item_name.to_ascii_lowercase().replace(" ", "").as_bytes(),
            item.url_name.as_bytes(),
        )?;
        num += 1;
        Result::<_, sled::Error>::Ok(())
    })?;

    db.flush_async().await?;

    Ok(num)
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum OrderType {
    #[serde(rename = "sell")]
    Sell,
    #[serde(rename = "buy")]
    Buy,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum UserStatus {
    #[serde(rename = "ingame")]
    InGame,
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub ingame_name: CompactStr,
    pub status: UserStatus,
    pub reputation: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    pub platinum: i32,
    pub quantity: i32,
    pub order_type: OrderType,
    pub region: CompactStr,
    pub visible: bool,
    pub user: User,
    pub mod_rank: Option<i32>,
}

pub async fn wm_item(url_name: &str) -> anyhow::Result<Vec<Order>> {
    #[derive(Deserialize, Debug, Clone)]
    struct Body {
        payload: Payload,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Payload {
        orders: Vec<Order>,
    }

    let payload = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(format!("{}/items/{}/orders", wm_api_url!(), url_name))
        .header("Platform", "pc")
        .send()
        .await?
        .json::<Body>()
        .await?
        .payload;

    Ok(payload.orders)
}

pub async fn update_rivens_db(db: Arc<sled::Db>) -> anyhow::Result<u32> {
    #[derive(Deserialize, Debug, Clone)]
    struct Body {
        payload: Payload,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Payload {
        items: Vec<Item>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Item {
        url_name: CompactStr,
        item_name: CompactStr,
    }

    let payload = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(concat!(wm_api_url!(), "/riven/items"))
        .header("Language", "zh-hans")
        .send()
        .await?
        .json::<Body>()
        .await?
        .payload;

    let mut num = 0u32;
    payload.items.into_iter().try_for_each(|item| {
        db.insert(
            item.item_name.to_ascii_lowercase().replace(" ", "").as_bytes(),
            item.url_name.as_bytes(),
        )?;
        num += 1;
        Result::<_, sled::Error>::Ok(())
    })?;

    db.flush_async().await?;

    Ok(num)
}

#[derive(Deserialize, Debug, Clone)]
pub struct Auction {
    pub buyout_price: Option<i32>,
    pub starting_price: i32,
    pub private: bool,
    pub visible: bool,
    pub item: AuctionItem,
    pub closed: bool,
    pub is_direct_sell: bool,
    pub owner: User,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AuctionItem {
    pub name: CompactStr,
    pub mastery_level: i32,
    pub mod_rank: i32,
    pub polarity: Polarity,
    pub re_rolls: i32,
    pub attributes: Vec<Attribute>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Attribute {
    pub positive: bool,
    pub value: f64,
    pub url_name: CompactStr,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Polarity {
    #[serde(rename = "naramon")]
    Naramon,
    #[serde(rename = "madurai")]
    Madurai,
    #[serde(rename = "vazarin")]
    Vazarin,
}

impl Polarity {
    pub fn nickname(&self) -> &'static str {
        match self {
            Polarity::Naramon => "-",
            Polarity::Madurai => "r",
            Polarity::Vazarin => "盾",
        }
    }
}

pub async fn wm_riven(
    url_name: &str,
    positive_stats: &str,
    negative_stats: &str,
) -> anyhow::Result<Vec<Auction>> {
    #[derive(Deserialize, Debug, Clone)]
    struct Body {
        payload: Payload,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct Payload {
        auctions: Vec<Auction>,
    }

    let positive_stats = if positive_stats.is_empty() {
        String::new()
    } else {
        format!("&positive_stats={}", positive_stats)
    };

    let negative_stats = if negative_stats.is_empty() {
        String::new()
    } else {
        format!("&negative_stats={}", negative_stats)
    };

    let payload = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()?
        .get(
            format!(
                "{}/auctions/search?type=riven&weapon_url_name={}&sort_by=price_asc{}{}",
                wm_api_url!(),
                url_name,
                positive_stats,
                negative_stats
            )
        )
        .header("Platform", "pc")
        .send()
        .await?
        .json::<Body>()
        .await?
        .payload;

    Ok(payload.auctions)
}
