export async function POST(
	request: Request,
	{ params }: { params: { name: string } },
) {
	let in_json;
	try {
		in_json = await request.json();
	} catch (e) {
		return new Response("", {
			status: 500,
			statusText: `Bad input json: ${e}`,
		});
	}

	let res;
	try {
		res = await fetch(`http://localhost:3030/pipelines/${params.name}/run`, {
			method: "POST",
			body: JSON.stringify(in_json),
			headers: request.headers,
		});
	} catch (err) {
		return new Response(`Could not reach server. Reason:\n${err}`, {
			status: 500,
			statusText: `Could not reach server`,
		});
	}

	if (!res.ok) {
		let text = await res.text();
		return new Response("", {
			status: 500,
			statusText: `Server returned error ${res.status} (${text})`,
		});
	}

	return new Response("", {
		status: 200,
	});
}
