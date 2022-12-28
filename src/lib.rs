extern crate wgpu;
extern crate pollster;
extern crate image;


pub mod state;

use state::State;

pub async fn run(){
  let mut st = State::new(1024,1024).await;
  st.render().await
}