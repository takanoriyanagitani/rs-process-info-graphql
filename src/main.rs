use actix_web::{App, HttpResponse, HttpServer, Result, guard, web};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use rs_process_info_graphql::{ProcessSchema, Query};

async fn index(schema: web::Data<ProcessSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = ProcessSchema::build(
        Query,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .finish();

    // Save schema to file
    std::fs::write("schema.graphql", schema.sdl()).expect("Failed to write schema");

    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    println!("Listening on {}", listen_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind(&listen_addr)?
    .run()
    .await
}
