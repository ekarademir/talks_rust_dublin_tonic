use std::collections::HashMap;
use std::sync::Arc;

use tonic::codegen::http::request;
use tonic::{
    transport::Server,
    Request,
    Response,
    Status
};
use tokio_stream::wrappers::ReceiverStream;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use log::{info, debug};

use chat::{
    chat_server::{Chat, ChatServer},
    Member,
    JoinResult,
    JoinResponse,
    After,
    ChatMessage,
    NewChatMessage,
    CommitResult
};

pub mod chat {
    tonic::include_proto!("chat");
}

const MAX_MSG_LEN:usize = 50;

#[derive(Debug, Default)]
pub struct ChatService {
    room: Arc<RwLock<HashMap<u64, Member>>>,
    history: Arc<RwLock<Vec<ChatMessage>>>,
}

impl ChatService {
    async fn add_message(&self, msg: NewChatMessage) -> Option<usize> {
        match self.get_username(msg.token).await {
            Some(username) => {
                let mut history = self.history.write().await;
                let idx = history.len() + 1;
                history.push(ChatMessage {
                    time: idx as u64,
                    username,
                    value: msg.value
                });
                Some(idx)
            },
            None => None
        }
    }

    async fn get_username(&self, token: u64) -> Option<String> {
        let room = self.room.read().await;
        let user = match room.get(&token) {
            Some(member) => Some(member.username.clone()),
            None => None
        };
        user
    }

    async fn add_user(&self, user: Member) -> u64 {
        let mut room = self.room.write().await;
        let token = (room.len() + 1) as u64;
        room.insert(token, user);
        token
    }

    async fn has_member(&self, member: &Member) -> bool {
        let room = self.room.read().await;
        for existing_member in room.values() {
            if existing_member.username == member.username {
                return true;
            }
        }
        false
    }
}

#[tonic::async_trait]
impl Chat for ChatService {
    async fn join(&self, request: Request<Member>) -> Result<Response<JoinResult>, Status> {
        let member = request.get_ref();
        let result;
        debug!("Joining {:?}", member.username);
        if self.has_member(member).await {
            debug!("Denied");
            result = JoinResult {
                token: 0,
                response: JoinResponse::Denied as i32
            };
        } else {
            debug!("Accepted");
            let token = self.add_user(member.to_owned()).await;
            result = JoinResult {
                token,
                response: JoinResponse::Accepted as i32
            };
        }

        Ok(Response::new(result))
    }

    type ChatLogStream = ReceiverStream<Result<ChatMessage, Status>>;
    async fn chat_log(&self, request: Request<After>) -> Result<Response<Self::ChatLogStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        if let Some(username) = self.get_username(request.get_ref().token).await {
            let history = self.history.clone();
            let after = request.get_ref().value;
            tokio::spawn(async move {
                info!("Sending messages that are after {:?} for user {:?}", after, username);
                for msg in history.read().await.iter() {
                    tx.send(Ok(msg.clone())).await.unwrap();
                }
            });
        } else {
            tx.send(Err(Status::unauthenticated("User does not exist"))).await.unwrap();
        }
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn commit(
        &self,
        request: Request<NewChatMessage>,
    ) -> Result<Response<CommitResult>, Status> {
        let new_msg = request.get_ref();
        if new_msg.value.len() > MAX_MSG_LEN {
            return Err(Status::cancelled("Message too long"));
        }
        if let Some(time) = self.add_message(request.get_ref().to_owned()).await {
            return Ok(Response::new(CommitResult{time: time as u64}));
        }
        Err(Status::unauthenticated("User does not exist"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    env_logger::Builder::new()
        .filter_module(
            "server", log::LevelFilter::Debug
        ).init();

    let room = Arc::new(RwLock::new(HashMap::new()));
    let history = Arc::new(RwLock::new(Vec::new()));


    let chat_service = ChatService {
        room,
        history,
    };

    let user1 = chat_service.add_user(Member {
        username: "user1".to_string(),
        password: "pass1".to_string(),
    }).await;

    let user2 = chat_service.add_user(Member {
        username: "user2".to_string(),
        password: "pass2".to_string(),
    }).await;

    chat_service.add_message(NewChatMessage {
        token: user1,
        value: "Hi!".to_string()
    }).await;

    chat_service.add_message(NewChatMessage {
        token: user2,
        value: "Hello sir!".to_string()
    }).await;


    let addr = "[::1]:10000".parse()?;
    let service = ChatServer::new(chat_service);

    info!("Listening at {:?}", addr);

    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, cleanup())
        .await?;

    Ok(())
}

async fn cleanup(){
    signal::ctrl_c().await.unwrap();
    log::info!("Good bye!");
}
