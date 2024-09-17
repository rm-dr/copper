import createClient from "openapi-fetch";
import { paths } from "./openapi";

export const edgeclient = createClient<paths>({
	baseUrl: "/api/",
});

/*
const client = createClient<paths>({ baseUrl })
client.use({
  async onRequest(req, _options) {
    req.headers.set('Authorization', `Bearer ${token}`)
    return req
  },
})
*/
