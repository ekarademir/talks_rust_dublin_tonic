use tonic::{
    transport::Server,
    Request,
    Response,
    Status
};
use tokio_stream::wrappers::ReceiverStream;
use tokio::signal;
use tokio::sync::mpsc;

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

#[derive(Debug, Default)]
pub struct ChatService {}

#[tonic::async_trait]
impl Chat for ChatService {
    async fn join(&self, request: Request<Member>) -> Result<Response<JoinResult>, Status> {
        let result = JoinResult {
            token: 123,
            response: JoinResponse::Accepted as i32
        };
        Ok(Response::new(result))
    }

    type ChatLogStream = ReceiverStream<Result<ChatMessage, Status>>;
    async fn chat_log(&self, request: Request<After>) -> Result<Response<Self::ChatLogStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let msg = ChatMessage {
            time: 10,
            username: "john".to_string(),
            value: "Hello".to_string(),
        };
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn commit(
        &self,
        request: Request<NewChatMessage>,
    ) -> Result<Response<CommitResult>, Status> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let chat_service = ChatService {};
    let addr = "[::1]:10000".parse()?;

    let service = ChatServer::new(chat_service);

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
