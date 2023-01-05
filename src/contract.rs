use app_io::*;
use gmeta::Metadata;
use gstd::{
    errors::{ContractError, Result as GstdResult},
    msg,
    prelude::*,
    util, ActorId, MessageId,
};
use hashbrown::HashMap;

static mut STATE: Option<HashMap<ActorId, u128>> = None;

fn static_mut_state() -> &'static mut HashMap<ActorId, u128> {
    unsafe { STATE.get_or_insert(Default::default()) }
}

#[no_mangle]
extern "C" fn handle() {
    process_handle()
        .expect("Failed to load, decode, encode, or reply with `PingPong` from `handle()`")
}

fn process_handle() -> Result<(), ContractError> {
    let payload = msg::load()?;

    if let PingPong::Ping = payload {
        let pingers = static_mut_state();

        pingers
            .entry(msg::source())
            .and_modify(|ping_count| *ping_count = ping_count.saturating_add(1))
            .or_insert(1);

        reply(PingPong::Pong)?;
    }

    Ok(())
}

fn common_state() -> <AppMetadata as Metadata>::State {
    static_mut_state()
        .iter()
        .map(|(pinger, ping_count)| (*pinger, *ping_count))
        .collect()
}

#[no_mangle]
extern "C" fn meta_state() -> *const [i32; 2] {
    let query = msg::load().expect("Failed to load or decode `AppStateQuery`");
    let state = common_state();

    let reply = match query {
        AppStateQuery::AllState => AppStateQueryReply::AllState(state),
        AppStateQuery::Pingers => AppStateQueryReply::Pingers(app_io::pingers(state)),
        AppStateQuery::PingCount(actor) => {
            AppStateQueryReply::PingCount(app_io::ping_count(state, actor))
        }
    };

    util::to_leak_ptr(reply.encode())
}

#[no_mangle]
extern "C" fn state() {
    reply(common_state())
        .expect("Failed to encode or reply with `<AppMetadata as Metadata>::State` from `state()`");
}

#[no_mangle]
extern "C" fn metahash() {
    reply(include!("../.metahash"))
        .expect("Failed to encode or reply with `[u8; 32]` from `metahash()`");
}

fn reply(payload: impl Encode) -> GstdResult<MessageId> {
    msg::reply(payload, 0)
}

gstd::metadata! {
    title: "App",
    handle:
        input: PingPong,
        output: PingPong,
    state:
        input: AppStateQuery,
        output: AppStateQueryReply,
}