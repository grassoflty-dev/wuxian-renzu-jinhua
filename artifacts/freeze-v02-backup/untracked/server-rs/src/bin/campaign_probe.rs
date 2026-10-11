//! Local development transport for the same Rust campaign state/actions as desktop IPC.
//!
//! This probe is deliberately memory-only. It never opens, reads, or writes a save file.
use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    panic::{catch_unwind, AssertUnwindSafe},
};

use serde_json::{json, Value};
use wuxian_horror_ch1::campaign::{self, CampaignState};

const MAX_LINE_BYTES: usize = 64 * 1024;

fn error_reply(message: impl Into<String>) -> Value {
    json!({ "error": message.into() })
}

fn handle_request(state: &mut CampaignState, line: &str) -> Value {
    if line.trim().is_empty() {
        return error_reply("request 不能为空");
    }

    let request = match serde_json::from_str::<Value>(line) {
        Ok(request) => request,
        Err(error) => return error_reply(format!("JSON 无效：{error}")),
    };
    let request = match request.as_object() {
        Some(request) => request,
        None => return error_reply("request 必须是 JSON object"),
    };

    let action = match request.get("action") {
        None => "view",
        Some(value) => match value.as_str() {
            Some(action) if !action.trim().is_empty() => action,
            Some(_) => return error_reply("action 不能为空"),
            None => return error_reply("action 必须是 string"),
        },
    };
    if action.len() > 128 {
        return error_reply("action 过长");
    }

    let args = request.get("args").cloned().unwrap_or_else(|| json!({}));
    if !args.is_object() {
        return error_reply("args 必须是 JSON object");
    }

    let reply = catch_unwind(AssertUnwindSafe(|| {
        if action == "view" {
            campaign::view(state)
        } else {
            campaign::act(state, action, &args)
        }
    }));
    match reply {
        Ok(reply) => {
            eprintln!("[campaign-probe] action={action} state={reply}");
            reply
        }
        Err(_) => error_reply("campaign action panic；状态未返回"),
    }
}

fn main() {
    eprintln!("[campaign-probe] memory_only=true; 不读写真实存档，仅供本地 QA。");
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = BufWriter::new(io::stdout());
    let mut line = String::new();
    let mut state = CampaignState::default();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("[campaign-probe] stdin 读取失败：{error}");
                break;
            }
        }

        let reply = if line.len() > MAX_LINE_BYTES {
            error_reply("request 行超过 64 KiB 限制")
        } else {
            handle_request(&mut state, &line)
        };
        if writeln!(stdout, "{reply}")
            .and_then(|_| stdout.flush())
            .is_err()
        {
            eprintln!("[campaign-probe] stdout 已关闭，停止服务。");
            break;
        }
    }
}
