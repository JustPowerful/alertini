use axum::{Router, routing::{get, post}};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/foo", get(get_foo) )
        .route("/foo", post(post_foo));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    return "Hello World";
}

async fn get_foo() -> &'static str {
    return "Hello Foo";
}

async fn post_foo() -> &'static str {
    return "Post Foo";
}


