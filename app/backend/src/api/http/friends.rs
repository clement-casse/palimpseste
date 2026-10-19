//! The routes of friends (ADR BE-FRIENDS) and of the invitations that make
//! them (ADR BE-INVITATIONS). An invitation's token is a secret: it travels
//! in JSON bodies, never in a URL, which the request trace logs. A friend is
//! named by their friendship; their characters are read, never changed.

use axum::{
    Json, Router,
    extract::{Path as UrlPath, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use super::{AppState, CharacterView, Rejection, SignedIn};
use crate::{
    CharacterId, Friend, FriendshipId, Invitation, InvitationId, InvitationStatus, InvitationToken,
    ReceivedInvitation,
};

/// The routes of friends and invitations, under `/api`.
pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/me/invitations",
            get(list_invitations).post(create_invitation),
        )
        .route("/me/invitations/{invitation_id}", delete(revoke_invitation))
        .route("/invitations/lookup", post(lookup_invitation))
        .route("/me/friends", get(list_friends).post(accept_invitation))
        .route(
            "/me/friends/{friend_id}",
            get(get_friend).delete(remove_friend),
        )
        .route("/me/friends/{friend_id}/characters", get(friend_characters))
        .route(
            "/me/friends/{friend_id}/characters/{character_id}",
            get(friend_character),
        )
}

/// A friend, named by their friendship: never their user id.
#[derive(Debug, Serialize)]
struct FriendView {
    id: i64,
    nickname: String,
    profile: String,
}

impl From<Friend> for FriendView {
    fn from(friend: Friend) -> Self {
        Self {
            id: friend.friendship.get(),
            nickname: friend.nickname.0,
            profile: friend.profile.0,
        }
    }
}

/// An invitation of the signed-in user, waiting to be accepted.
#[derive(Debug, Serialize)]
struct InvitationView {
    id: i64,
    /// When it expires, in seconds since the Unix epoch.
    expires: i64,
}

impl From<Invitation> for InvitationView {
    fn from(invitation: Invitation) -> Self {
        Self {
            id: invitation.id.get(),
            expires: invitation.expires_at.seconds(),
        }
    }
}

/// A new invitation, with its token: the only time it is sent.
#[derive(Debug, Serialize)]
struct CreatedInvitationView {
    id: i64,
    token: String,
    expires: i64,
}

/// An invitation as the user who opened its link sees it: who sent it, and
/// what accepting it would do.
#[derive(Debug, Serialize)]
struct ReceivedInvitationView {
    nickname: String,
    profile: String,
    /// `open`, `own` or `friends`.
    status: &'static str,
    /// The friendship, when they are already friends.
    friend: Option<i64>,
}

impl From<ReceivedInvitation> for ReceivedInvitationView {
    fn from(ReceivedInvitation { inviter, status }: ReceivedInvitation) -> Self {
        let (status, friend) = match status {
            InvitationStatus::Open => ("open", None),
            InvitationStatus::Own => ("own", None),
            InvitationStatus::Friends(friendship) => ("friends", Some(friendship.get())),
        };
        Self {
            nickname: inviter.nickname.0,
            profile: inviter.profile.0,
            status,
            friend,
        }
    }
}

/// The token of an invitation link.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InvitationLink {
    token: String,
}

impl InvitationLink {
    fn token(&self) -> InvitationToken {
        InvitationToken::from_link(&self.token)
    }
}

async fn list_invitations(
    signed_in: SignedIn,
    State(state): State<AppState>,
) -> Result<Json<Vec<InvitationView>>, Rejection> {
    let invitations = state.service.invitations(&signed_in.context()).await?;
    Ok(Json(invitations.into_iter().map(Into::into).collect()))
}

async fn create_invitation(
    signed_in: SignedIn,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<CreatedInvitationView>), Rejection> {
    let (token, invitation) = state
        .service
        .create_invitation(&signed_in.context())
        .await?;
    let created = CreatedInvitationView {
        id: invitation.id.get(),
        token: token.as_str().to_owned(),
        expires: invitation.expires_at.seconds(),
    };
    Ok((StatusCode::CREATED, Json(created)))
}

async fn revoke_invitation(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
) -> Result<StatusCode, Rejection> {
    state
        .service
        .revoke_invitation(&signed_in.context(), InvitationId::from(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn lookup_invitation(
    signed_in: SignedIn,
    State(state): State<AppState>,
    Json(link): Json<InvitationLink>,
) -> Result<Json<ReceivedInvitationView>, Rejection> {
    let received = state
        .service
        .received_invitation(&signed_in.context(), &link.token())
        .await?;
    Ok(Json(received.into()))
}

async fn accept_invitation(
    signed_in: SignedIn,
    State(state): State<AppState>,
    Json(link): Json<InvitationLink>,
) -> Result<(StatusCode, Json<FriendView>), Rejection> {
    let friend = state
        .service
        .accept_invitation(&signed_in.context(), &link.token())
        .await?;
    Ok((StatusCode::CREATED, Json(friend.into())))
}

async fn list_friends(
    signed_in: SignedIn,
    State(state): State<AppState>,
) -> Result<Json<Vec<FriendView>>, Rejection> {
    let friends = state.service.friends(&signed_in.context()).await?;
    Ok(Json(friends.into_iter().map(Into::into).collect()))
}

async fn get_friend(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
) -> Result<Json<FriendView>, Rejection> {
    let friend = state
        .service
        .friend(&signed_in.context(), FriendshipId::from(id))
        .await?;
    Ok(Json(friend.into()))
}

async fn remove_friend(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
) -> Result<StatusCode, Rejection> {
    state
        .service
        .remove_friend(&signed_in.context(), FriendshipId::from(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn friend_characters(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
) -> Result<Json<Vec<CharacterView>>, Rejection> {
    let characters = state
        .service
        .friend_characters(&signed_in.context(), FriendshipId::from(id))
        .await?;
    Ok(Json(characters.into_iter().map(Into::into).collect()))
}

async fn friend_character(
    signed_in: SignedIn,
    State(state): State<AppState>,
    UrlPath((id, character)): UrlPath<(i64, i64)>,
) -> Result<Json<CharacterView>, Rejection> {
    let character = state
        .service
        .friend_character(
            &signed_in.context(),
            FriendshipId::from(id),
            CharacterId::from(character),
        )
        .await?;
    Ok(Json(character.into()))
}

#[cfg(test)]
mod tests {
    use axum::http::{Method, header};
    use serde_json::{Value, json};

    use super::{
        super::tests::{App, app, lhagva, send_with, with_character},
        *,
    };

    /// Sends a request with `cookie`, and reads its JSON body, if any.
    async fn send(
        app: &App,
        cookie: &str,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let (status, _, bytes) = send_with(&app.router, Some(cookie), method, uri, &[], body).await;
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, json)
    }

    /// A new invitation of the user of `cookie`: its token.
    async fn invite(app: &App, cookie: &str) -> String {
        let (status, created) = send(app, cookie, Method::POST, "/api/me/invitations", None).await;
        assert_eq!(status, StatusCode::CREATED);
        created["token"].as_str().unwrap().to_owned()
    }

    /// Ana's friendship with Ben, through an invitation of Ana's: its id.
    async fn befriend(app: &App) -> i64 {
        let link = json!({ "token": invite(app, &app.ana).await });
        let (status, ana) = send(app, &app.ben, Method::POST, "/api/me/friends", Some(link)).await;
        assert_eq!(status, StatusCode::CREATED);
        ana["id"].as_i64().unwrap()
    }

    #[tokio::test]
    async fn an_invitation_link_makes_friends_of_both() {
        let app = app("friends").await;
        let (ana, ben) = (app.ana.as_str(), app.ben.as_str());
        send(
            &app,
            ana,
            Method::PATCH,
            "/api/me",
            Some(json!({"profile": "Joue une barbare."})),
        )
        .await;
        let link = json!({ "token": invite(&app, ana).await });
        let (_, pending) = send(&app, ana, Method::GET, "/api/me/invitations", None).await;
        assert_eq!(pending.as_array().unwrap().len(), 1);
        assert!(pending[0]["expires"].as_i64().unwrap() > 0);
        assert!(pending[0].get("token").is_none());

        // Its sender cannot use it.
        let lookup = "/api/invitations/lookup";
        let (status, own) = send(&app, ana, Method::POST, lookup, Some(link.clone())).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(own["status"], "own");
        let (status, _) = send(
            &app,
            ana,
            Method::POST,
            "/api/me/friends",
            Some(link.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

        let (status, received) = send(&app, ben, Method::POST, lookup, Some(link.clone())).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            received,
            json!({"nickname": "Ana", "profile": "Joue une barbare.", "status": "open", "friend": null})
        );
        let (status, ana_for_ben) = send(
            &app,
            ben,
            Method::POST,
            "/api/me/friends",
            Some(link.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let friend = ana_for_ben["id"].as_i64().unwrap();
        assert_eq!(
            ana_for_ben,
            json!({"id": friend, "nickname": "Ana", "profile": "Joue une barbare."})
        );

        // Used up.
        let (status, _) = send(
            &app,
            app.cy.as_str(),
            Method::POST,
            "/api/me/friends",
            Some(link),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (_, pending) = send(&app, ana, Method::GET, "/api/me/invitations", None).await;
        assert_eq!(pending, json!([]));

        let (_, friends) = send(&app, ben, Method::GET, "/api/me/friends", None).await;
        assert_eq!(friends, json!([ana_for_ben]));
        let (_, friends) = send(&app, ana, Method::GET, "/api/me/friends", None).await;
        assert_eq!(
            friends,
            json!([{"id": friend, "nickname": "Ben", "profile": ""}])
        );
        let (status, ben_for_ana) = send(
            &app,
            ana,
            Method::GET,
            &format!("/api/me/friends/{friend}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ben_for_ana["nickname"], "Ben");

        // A link between friends says so.
        let again = json!({ "token": invite(&app, ana).await });
        let (_, received) = send(&app, ben, Method::POST, lookup, Some(again.clone())).await;
        assert_eq!(received["status"], "friends");
        assert_eq!(received["friend"], friend);
        let (status, _) = send(&app, ben, Method::POST, "/api/me/friends", Some(again)).await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn friends_read_each_others_characters_and_change_none() {
        let app = with_character("friend-characters").await;
        let friend = befriend(&app).await;
        let ben = app.ben.as_str();
        let characters = format!("/api/me/friends/{friend}/characters");
        let (status, list) = send(&app, ben, Method::GET, &characters, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(list[0]["name"], "Brune");
        let (status, brune) = send(&app, ben, Method::GET, &format!("{characters}/1"), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(brune["document"]["engine"], "fantasy");
        // Ben has none.
        let (_, list) = send(&app, &app.ana, Method::GET, &characters, None).await;
        assert_eq!(list, json!([]));
        let (status, _) = send(
            &app,
            &app.ana,
            Method::GET,
            &format!("{characters}/1"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        for (method, uri, body) in [
            (Method::GET, "/api/characters/1", None),
            (
                Method::PATCH,
                "/api/characters/1",
                Some(json!({"name": "X"})),
            ),
            (Method::DELETE, "/api/characters/1", None),
        ] {
            let (status, _) = send(&app, ben, method, uri, body).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        }
        let (status, _, _) = send_with(
            &app.router,
            Some(ben),
            Method::PUT,
            "/api/characters/1/document",
            &[(header::IF_MATCH, "\"1\"")],
            Some(lhagva()),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn either_friend_ends_the_friendship_for_both() {
        let app = with_character("unfriend").await;
        let friend = befriend(&app).await;
        let reaches = [
            format!("/api/me/friends/{friend}"),
            format!("/api/me/friends/{friend}/characters"),
            format!("/api/me/friends/{friend}/characters/1"),
        ];
        // A stranger to both reaches nothing through their friendship.
        let cy = app.cy.as_str();
        for uri in &reaches {
            let (status, _) = send(&app, cy, Method::GET, uri, None).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        }
        let (status, _) = send(&app, cy, Method::DELETE, &reaches[0], None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = send(&app, &app.ben, Method::DELETE, &reaches[0], None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        for cookie in [&app.ana, &app.ben] {
            let (_, friends) = send(&app, cookie, Method::GET, "/api/me/friends", None).await;
            assert_eq!(friends, json!([]));
            for uri in &reaches {
                let (status, _) = send(&app, cookie, Method::GET, uri, None).await;
                assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
            }
        }
    }

    #[tokio::test]
    async fn a_new_invitation_gives_its_token_once() {
        let app = app("new-invitation").await;
        let ana = app.ana.as_str();
        let (status, first) = send(&app, ana, Method::POST, "/api/me/invitations", None).await;
        assert_eq!(status, StatusCode::CREATED);
        let token = first["token"].as_str().unwrap();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(first["id"].is_i64());
        assert!(first["expires"].as_i64().unwrap() > 0);
        let (_, second) = send(&app, ana, Method::POST, "/api/me/invitations", None).await;
        assert_ne!(second["token"], first["token"]);
        assert_ne!(second["id"], first["id"]);
        let (_, waiting) = send(&app, ana, Method::GET, "/api/me/invitations", None).await;
        assert_eq!(
            waiting,
            json!([
                {"id": first["id"], "expires": first["expires"]},
                {"id": second["id"], "expires": second["expires"]},
            ])
        );
    }

    #[tokio::test]
    async fn a_token_is_read_from_a_body_of_its_own() {
        let app = app("token-body").await;
        for uri in ["/api/invitations/lookup", "/api/me/friends"] {
            for body in [json!({}), json!({"token": 7})] {
                let (status, _) = send(&app, &app.ben, Method::POST, uri, Some(body.clone())).await;
                assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{uri} {body}");
            }
            let (status, _, _) = send_with(
                &app.router,
                Some(&app.ben),
                Method::POST,
                uri,
                &[(header::CONTENT_TYPE, "application/json")],
                None,
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{uri} without a body");
        }
    }

    #[tokio::test]
    async fn invitations_are_revoked_by_their_sender_only() {
        let app = app("invitations").await;
        let (ana, ben) = (app.ana.as_str(), app.ben.as_str());
        let token = invite(&app, ana).await;
        let (_, pending) = send(&app, ana, Method::GET, "/api/me/invitations", None).await;
        let revoke = format!("/api/me/invitations/{}", pending[0]["id"]);
        let (status, _) = send(&app, ben, Method::DELETE, &revoke, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = send(&app, ana, Method::DELETE, &revoke, None).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let link = json!({ "token": token });
        for uri in ["/api/invitations/lookup", "/api/me/friends"] {
            let (status, _) = send(&app, ben, Method::POST, uri, Some(link.clone())).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        }
        let (status, _) = send(
            &app,
            ben,
            Method::POST,
            "/api/me/friends",
            Some(json!({"token": token, "friend": 1})),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
