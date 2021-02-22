// #![cfg(feature = "tokio/sync")]
//! A simple example of hooking up stdin/stdout to a WebSocket stream.
//!
//! This example will connect to a server specified in the argument list and
//! then forward all data read on stdin to the server, printing out all data
//! received on stdout.
//!
//! Note that this is not currently optimized for performance, especially around
//! buffer management. Rather it's intended to show an example of working with a
//! client.
//!
//! You can use this example together with the `server` example.
use std::{env, time::Duration};

use future::join_all;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let connect_addr =
        env::args().nth(1).unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let (sender, mut receiver) = mpsc::channel::<String>(1);
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(async move {
        read_stdin(stdin_tx, &mut receiver).await;
    });

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    let sender_handle = async {
        loop {
            sender.send("from sender".to_owned()).await.ok();
            tokio::time::sleep(Duration::from_millis(500)).await
        }
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    // future::select(stdin_to_ws, ws_to_stdout).await;
    tokio::join!(future::select(stdin_to_ws, ws_to_stdout), sender_handle);
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
// async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
async fn read_stdin(
    tx: futures_channel::mpsc::UnboundedSender<Message>,
    receiver: &mut mpsc::Receiver<String>,
) {
    // loop {
    //     while let Ok(val) = receiver.recv() {

    //     }
    // }
    // let mut stdin = tokio::io::stdin();
    let interval_handle = async {
        loop {
            // let mut buf = vec![0; 1024];
            // let n = match stdin.read(&mut buf).await {
            //     Err(_) | Ok(0) => break,
            //     Ok(n) => n,
            // };
            // buf.truncate(n);
            // tx.unbounded_send(Message::binary(buf)).unwrap();
            tx.unbounded_send(Message::Text("aaaa".to_owned())).unwrap();
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    };
    let receiver_handle = async {
        loop {
            while let Some(val) = receiver.recv().await {
                tx.unbounded_send(Message::Text(val)).unwrap();
            }
        }
    };
    tokio::join!(interval_handle, receiver_handle);
}
