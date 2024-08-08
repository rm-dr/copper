import createClient from "openapi-fetch";
import { paths } from "./openapi";

export const APIclient = createClient<paths>({
	baseUrl: "/api/",
});
