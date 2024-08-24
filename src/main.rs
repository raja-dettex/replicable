
use futures::{stream, Sink, StreamExt};
use tokio::sync::mpsc::error::SendError;
use tokio_postgres::{Error, NoTls};
use std::pin::Pin;

use std::fmt::Debug;
use std::io::Write;

use std::task::{Context, Poll};

use serde::{Serialize , Deserialize};
pub mod sink;
use sink::sink::{Row, SinkWriter, WriteError, Writer};


// case - message is a type which implements writer trait. 
// message is a type which which we receive from a postgres source database and deserialize to this type,

#[derive(Deserialize, Debug, Serialize)]
struct Message { 
    message_id : i32,
    message: String 
}

impl Writer for Message { 
    fn write(&self, mut file : std::fs::File) -> Result<(), WriteError> {
        let bytes = serde_json::to_vec(&self).unwrap();
        let written = file.write_all(&bytes).map_err(|_| WriteError{});
        let _ = file.write_all(b"\n");
        let _ = file.flush();
        written
    }
}



pub async fn async_connector<T>() -> Result<(), Error> 
where T : Serialize + for <'a> serde::de::Deserialize<'a> +  Debug + Writer
{ 
    let (client, mut conn) = tokio_postgres::connect("host=localhost user=demo password=demo dbname=demo_db", NoTls).await?;
    println!("conn : {:?}", conn.parameter("host"));
    
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<tokio_postgres::AsyncMessage>();
    println!("here");

    // by polling the futures of the connection socket stream we create a stream of futures which is the way forward, 
    let stream = stream::poll_fn(move |cx | conn.poll_message(cx).map_err(|err| panic!("{err}")) );
    println!("before spin");
    // afterwards the task to forward the stream to SenderSink which implements Sink is spawned, 
    let forward_task = tokio::spawn(async move { 
        println!("spinning");
        stream.forward(SenderSink(tx)).await.expect("send") 
    });
    println!("listening..");
    client.batch_execute("LISTEN activity_channel;").await.expect("litening error");
    
    while let Some(msg) = rx.recv().await { 
        let tokio_postgres::AsyncMessage::Notification(noti) = msg else { 
            todo!()
        };
        let p = noti.payload();
        let message = Row::deserialize(p).unwrap();
        
        //let str = String::from_utf8(buf.to_vec()).expect("not valid utf-8");
        let file =  std::fs::OpenOptions::new().create(true).write(true).append(true).open("./output.bin").unwrap();
        let file_sink = SinkWriter::<T>{t: message};
        println!("writing..");
        if let Err(err) = file_sink.write(file) { 
            eprintln!("write error : {err:?}")
        }
        //println!("msg : {message:#?}");
    }
    
    forward_task.await.expect("failed");    
    Ok(())
}


// SenderSink is a wrapper around the tokio's unbounded sender , this wrapper implements sink
struct SenderSink<T>(tokio::sync::mpsc::UnboundedSender<T>);


impl<T> Sink<T> for SenderSink<T> {
    type Error = SendError<T>;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> { 
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.get_mut().0.send(item)?;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}





#[tokio::main]
async fn main() -> Result<(), Error>{
    async_connector::<Message>().await
}
