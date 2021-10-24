use clap::{
    App,
    AppSettings,
    Arg,
    SubCommand,
};
use tonic::Request;

use chat::chat_client::ChatClient;
use chat::{
    Member,
    JoinResponse,
    NewChatMessage,
};

pub mod chat {
    tonic::include_proto!("chat");
}

const DEFAULT_SERVER:&str = "http://[::1]:10000";

#[tokio::main]
async fn main() {
    let matches = App::new("client")
            .setting(AppSettings::SubcommandRequired)
            .subcommand(SubCommand::with_name("join")
                .about("Join a chat server")
                .arg(Arg::with_name("server")
                    .short("s")
                    .long("server")
                    .value_name("SERVER")
                    .default_value(DEFAULT_SERVER)
                    .required(false)
                )
                .arg(Arg::with_name("username")
                    .short("u")
                    .long("username")
                    .value_name("USERNAME")
                    .required(true)
                )
                .arg(Arg::with_name("password")
                    .short("p")
                    .long("password")
                    .value_name("PASSWORD")
                    .required(true)
                )
                .help("Join a server to obtain token")
            )
            .subcommand(SubCommand::with_name("send")
                .about("send a message to the server")
                .arg(Arg::with_name("server")
                    .short("s")
                    .long("server")
                    .value_name("SERVER")
                    .default_value(DEFAULT_SERVER)
                    .required(false)
                )
                .arg(Arg::with_name("token")
                    .short("t")
                    .long("token")
                    .value_name("TOKEN")
                    .required(true)
                )
                .arg(Arg::with_name("message")
                    .short("m")
                    .long("message")
                    .value_name("MESSAGE")
                    .required(true)
                )
                .help("Send a message to the server")
            )
            .get_matches();

    if let Some(matches) = matches.subcommand_matches("join") {
        let server_addr = matches.value_of("server").unwrap().to_string();
        let username = matches.value_of("username").unwrap().to_string();
        let password = matches.value_of("password").unwrap().to_string();
        println!("Connecting {}", server_addr);
        if let Ok(mut client) = ChatClient::connect(server_addr).await {
            println!("Obtaining token");
            match client.join(Request::new(Member {
                username,
                password
            })).await {
                Ok(join_result) => {
                    if join_result.get_ref().response == JoinResponse::Accepted as i32 {
                        println!("Token: {}", join_result.get_ref().token);
                    }
                    if join_result.get_ref().response == JoinResponse::Denied as i32 {
                        println!("Access denied");
                    }
                },
                Err(e) => println!("Can't obtain token {:?}", e)
            }
        } else {
            println!("Can't connect");
        };
    }

    if let Some(matches) = matches.subcommand_matches("send") {
        let server_addr = matches.value_of("server").unwrap().to_string();
        let token = matches.value_of("token").unwrap().parse().unwrap();
        let message = matches.value_of("message").unwrap().to_string();
        println!("Connecting {}", server_addr);
        if let Ok(mut client) = ChatClient::connect(server_addr).await {
            println!("Sending message");
            match client.commit(Request::new(NewChatMessage {
                token,
                value: message
            })).await {
                Ok(commit_result) => println!("Last message time {:?}", commit_result.get_ref().time),
                Err(e) => println!("Can't send message {:?}", e)
            }
        } else {
            println!("Can't connect");
        };
    }
}
