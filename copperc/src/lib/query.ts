import { useQuery } from "@tanstack/react-query";
import { edgeclient } from "./api/client";

/**
 * Query `/user/me` and return user info.
 * Redirects to login page if we are not logged in.
 */
export function useUserInfoQuery() {
	return useQuery({
		queryKey: ["user/me"],
		queryFn: async () => {
			const res = await edgeclient.GET("/user/me");
			if (res.response.status !== 200) {
				location.replace("/");
			}
			return res;
		},
		staleTime: 5,
	});
}
