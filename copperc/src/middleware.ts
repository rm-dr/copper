import { NextRequest, NextResponse } from "next/server";

export const config = {
	matcher: ["/api/:path*"],
};

// EDGED_ADDR should look like "http://copperd:80"

// We use middleware instead of nextjs rewrites, since those cannot load envvars at runtime
export function middleware(request: NextRequest) {
	if (process.env.EDGED_ADDR === undefined || process.env.EDGED_ADDR === "") {
		console.error("EDGED_ADDR has not been set, cannot rewrite api!");
		console.error(
			"Set the EDGED_ADDR environment variable to resolve this problem.",
		);
		return;
	}

	if (request.nextUrl.pathname.startsWith("/api/")) {
		const path_stripped = request.nextUrl.pathname.slice(4);
		const target_url = `${process.env.EDGED_ADDR}${path_stripped}${request.nextUrl.search}`;
		return NextResponse.rewrite(new URL(target_url), { request });
	}
}
