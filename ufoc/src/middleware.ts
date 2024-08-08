import { NextRequest, NextResponse } from "next/server";

export function middleware(req: NextRequest) {
	const auth = req.cookies.get("authtoken")?.value;

	if (auth === undefined || auth === "") {
		// If not logged in, redirect everything to /login
		if (!req.nextUrl.pathname.startsWith("/login")) {
			const url = req.nextUrl.clone();
			url.pathname = "/login";
			return NextResponse.redirect(url, req.url);
		}
	} else {
		// If logged in, redirect away from /login to home page
		if (req.nextUrl.pathname.startsWith("/login")) {
			const url = req.nextUrl.clone();
			url.pathname = "/";
			return NextResponse.redirect(url, req.url);
		}
	}
}

export const config = {
	matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"],
};
