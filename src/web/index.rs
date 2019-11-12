use actix_identity::Identity;
use actix_web::{http, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;

use crate::core::{authentication_password, Account, Pool};
use crate::web::{LoggedAccount, WebResult};

#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub fn index(
    hb: web::Data<Handlebars>,
    logged_account: Option<LoggedAccount>,
    req: HttpRequest,
    pool: web::Data<Pool>,
) -> WebResult<HttpResponse> {
    match logged_account {
        None => {
            let data = json!({
                "error": req.query_string().contains("error")
            });
            let body = hb.render("index", &data)?;

            Ok(HttpResponse::Ok().body(body))
        }
        Some(account_id) => {
            let conn = &pool.get()?;
            let account = Account::get(&conn, &account_id.id);
            match account {
                Ok(account) => {
                    let data = json!({ "name": account.name.unwrap_or(account.id) });
                    let body = hb.render("home", &data)?;

                    Ok(HttpResponse::Ok().body(body))
                }
                Err(_) => Ok(HttpResponse::Found()
                    .header(http::header::LOCATION, "/logout")
                    .finish()),
            }
        }
    }
}

pub fn login(
    params: web::Form<LoginForm>,
    id: Identity,
    pool: web::Data<Pool>,
) -> WebResult<HttpResponse> {
    let conn = &pool.get()?;

    let login_result = authentication_password::get(conn, &params.username, &params.password);
    match login_result {
        Ok(account) => {
            let logged_account = serde_json::to_string(&LoggedAccount { id: account.id })?;
            id.remember(logged_account);

            Ok(HttpResponse::Found()
                .header(http::header::LOCATION, "/")
                .finish())
        }
        Err(_) => Ok(HttpResponse::Found()
            .header(http::header::LOCATION, "/?error")
            .finish()),
    }
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Found()
        .header(http::header::LOCATION, "/")
        .finish()
}
