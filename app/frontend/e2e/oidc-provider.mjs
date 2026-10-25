// A mock OIDC provider for the end-to-end tests: it signs anyone in at
// once, as one user, through the real Authorization Code flow (PKCE, state
// and nonce checked by the backend as with any provider).
import { OAuth2Server } from "oauth2-mock-server";

const server = new OAuth2Server();
await server.issuer.keys.generate("RS256");
server.service.on("beforeTokenSigning", (token) => {
  token.payload.preferred_username = "Testeur";
  token.payload.email = "testeur@example.test";
});
// The issuer the backend is configured with, which the tokens must name.
server.issuer.url = "http://127.0.0.1:9400";
await server.start(9400, "127.0.0.1");
console.log(`mock OIDC provider at ${server.issuer.url}`);
