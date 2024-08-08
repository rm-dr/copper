export const revalidate = 1;

export async function GET() {
	let res;
	try {
		res = await fetch("http://localhost:3030/status/runner");
	} catch (err) {
		return new Response(`Could not reach server. Reason:\n${err}`, {
			status: 500,
			statusText: `Could not reach server`,
		});
	}

	if (!res.ok) {
		let text = await res.text();
		return new Response(`Server returned error ${res.status}:\n${text}`, {
			status: 500,
			statusText: `Server returned error ${res.status}`,
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
