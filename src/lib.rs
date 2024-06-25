/// # forexpros-wss
/// 
/// Simple secured websocket (wss) client to forexpros.com stream--a realtime source for investing.com.
/// 
/// # Example
/// 
/// ```
/// use forexpros_wss::push;
/// 
/// fn main ( ) {
/// 	let pair_id = "945629";	// BTC/USD
/// 	//let pair_id = "8984";	// HK50 future
/// 	
/// 	let handler = |s| {
/// 		println ! ( "INFO: {:?}", s );
///		
///         // stop the stream
///		    Err (())
/// 	};
/// 
/// 	let runtime = tokio::runtime::Runtime::new ( )
/// 		.unwrap ( );
/// 
/// 	let stream = push::Stream::new ( pair_id.to_string ( ), &runtime, handler )
/// 		.expect ( "Failed to create stream" );
/// 	
/// 	println ! ("main: stream.pair_id={}", stream.pair_id);
/// 	println ! ("main: stream.handler={:?}", stream.stream_handle_spawn);
/// 	
/// 	runtime.block_on (
/// 		stream.stream_handle_spawn
/// 	).unwrap ( ).unwrap ( );
/// }
/// ```
/// 
/// 20:00 THA 03/12/2020
/// 
/// # References
/// 
///	https://docs.rs/tokio-tungstenite/0.12.0/tokio_tungstenite/
///	https://docs.rs/tungstenite/0.11.1/tungstenite/
///	https://docs.rs/tokio/0.3.5/tokio/
///	https://docs.serde.rs/serde_json/index.html
/// https://docs.rs/regex/1.4.2/regex/
///	https://github.com/websockets-rs/rust-websocket/issues/160
///	https://stackoverflow.com/questions/26946646/rust-package-with-both-a-library-and-a-binary/26946705#26946705
///	https://www.reddit.com/r/rust/comments/k5sb9o/tokio_block_onjoinhandle_freeze_randomly/
/// 
/// 23:44 THA 25/06/2024
/// 
/// Get tokio::runtime::Runtime as borrowed parameter instead.

pub mod push;
pub mod data;