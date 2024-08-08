export async function POST() {
	let res;
	try {
		res = await fetch("http://localhost:3030/upload/new", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
		});
	} catch (err) {
		return new Response(`Could not reach server. Reason:\n${err}`, {
			status: 500,
			statusText: `Could not reach server`,
		});
	}

	let out_json;
	try {
		out_json = await res.json();
	} catch (e) {
		return new Response(`Server returned bad json:\n${e}`, {
			status: 500,
			statusText: `Server returned bad json`,
		});
	}

	return Response.json(out_json);
}
