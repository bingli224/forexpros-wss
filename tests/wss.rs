
/// # Logic test: access to wss server
/// 
/// # Expectaion
/// 
/// json data from wss server.
/// 
/// # Raw events
/// 
/// Subscription to server
///     ["{\"_event\":\"bulk-subscribe\",\"tzID\":8,\"message\":\"pid-1058142:%%pid-8873:%%pid-8839:%%pid-169:%%pid-166:%%pid-14958:%%pid-44336:%%pid-8827:%%pid-1:%%pid-2:%%pid-3:%%pid-5:%%pid-7:%%pid-9:%%pid-10:%%pid-945629:%%pid-1065395:%%pid-8830:%%pid-8984:%%pidTechSumm-1:%%pidTechSumm-2:%%pidTechSumm-3:%%pidTechSumm-5:%%pidTechSumm-7:%%pidTechSumm-9:%%pidTechSumm-10:%%pidExt-1058142:%%isOpenExch-1053:%%isOpenExch-1:%%isOpenExch-2:%%isOpenPair-8873:%%isOpenPair-8839:%%isOpenPair-44336:%%isOpenPair-8827:%%cmt-1-5-1058142:%%domain-1:\"}"]
///     ["{\"_event\":\"UID\",\"UID\":0}"]
/// 
/// Keep-alive message to server
///     ["{\"_event\":\"heartbeat\",\"data\":\"h\"}"]
/// 
/// Keep-alive response from server
///     a["{\"_event\":\"heartbeat\",\"data\":\"h\"}"]
 
use tokio::{
    self,
    time,
};
use tokio_tungstenite;
use rand::Rng;
use futures::prelude::*;
use std::time::Duration;

#[test]
fn test_access_to_server ( /*pid: 'static &str*/ ) {
	let pair_id = "945629";	// BTC/USD
	//let pair_id = "8984";	// HK50 future

	let mut rnd = rand::thread_rng ( );

	let url = format ! ( "wss://stream2{:02}.forexpros.com/echo/{:03}/{:08}/websocket",
		//1 + rnd.gen::<u16> ( ) % 280,
		rnd.gen::<u8> ( ) % 100,
		rnd.gen::<u16> ( ) % 1000,
		rnd.gen::<u64> ( ) % 1_0000_0000
	);

	println ! ( "{}", &url );

	// https://stackoverflow.com/questions/61752896/how-to-create-a-dedicated-threadpool-for-cpu-intensive-work-in-tokio
	let rt_main = tokio::runtime::Runtime::new ( ).unwrap ( );
	let rt_heartbeat = rt_main
		.handle ( ).clone ( );

	rt_main
		.block_on ( async {
			let client = tokio_tungstenite::connect_async (
				&url
			)
			//.and_then ( |(mut stream, response)| async move {
			.then ( |stream_response| async move {
				stream_response.unwrap ( )
			} )
			.then ( |(mut stream, response)| async move {
				println ! ( "response: {:?}", response );
				if stream.next ( ).await.unwrap ( ).unwrap ( ).to_text ( ).unwrap ( ) == "o" {
					Ok ( stream.split ( ) )
				} else {
					Err ( () )
				}
			} )
			.and_then ( |(mut tx, mut rx)| async move {
				// TODO: react to the server
				tx.send ( format ! ( "[\"{{\\\"_event\\\":\\\"bulk-subscribe\\\",\\\"tzID\\\":\\\"8\\\",\\\"message\\\":\\\"pid-{}:\\\"}}\"]", &pair_id ).into ( ) )
					.await
					.expect ( "Expect tx.send(bulk-subscribe, tzID, pid) to server" )
					;
				tx.send ( "[\"{\\\"_event\\\":\\\"UID\\\",\\\"UID\\\":0}\"]".into ( ) )
					.await
					.expect ( "Expect tx.send(UID=0) to server" )
					;
				
				// send heartbeat responses to server
				rt_heartbeat
					.spawn ( async move {
						loop {
							tx.send ( "[\"{\\\"_event\\\":\\\"heartbeat\\\",\\\"data\\\":\\\"h\\\"}\"]".into ( ) )
								.await
								.expect ( "Expect tx.send(heartbeat) to server" )
								;
							print ! ( "." );
							time::sleep ( Duration::from_millis ( 3200u64 ) ).await;
						}
					} );
				
				println ! ( "rx.for_each()..." );
				
				// exit when got updated status of instrument
				let target = format ! ( r#""{}\\"#, pair_id );
				while let Some ( msg ) = rx.next ( ).await {
					let msg = msg.unwrap ( ).into_text ( ).unwrap ( );
					
					println ! ( "msg: {:?}", msg );

					if msg.contains ( &target ) {
						break;
					}
				}
						
				println ! ( "EOD: Found updated info" );
				Ok ( ( ) )
			} )
			.or_else ( |e| async move {
				println ! ( "Failed: {:?}", e );
				Err ( ( ) )
			} );
			
			client.await
		} ).unwrap ( );
}