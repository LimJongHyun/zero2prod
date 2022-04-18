use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body =
        serde_urlencoded::to_string([("name", "le guin"), ("email", "ursula_le_guin@gmail.com")])
            .expect("URL Encode error");

    Mock::given(path(&app.email_send_path))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let confirmation_link =
        app.get_confirmation_links(&app.email_server.received_requests().await.unwrap()[0]);
    let r = reqwest::get(confirmation_link).await.unwrap();
    assert_eq!(r.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body =
        serde_urlencoded::to_string([("name", "le guin"), ("email", "ursula_le_guin@gmail.com")])
            .expect("URL Encode error");

    Mock::given(path(&app.email_send_path))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let confirmation_link =
        app.get_confirmation_links(&app.email_server.received_requests().await.unwrap()[0]);
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}
