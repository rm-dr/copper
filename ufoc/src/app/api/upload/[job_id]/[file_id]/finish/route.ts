export async function POST(
	request: Request,
	{ params }: { params: { job_id: string; file_id: string } },
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
		res = await fetch(
			`http://localhost:3030/upload/${params.job_id}/${params.file_id}/finish`,
			{
				method: "POST",
				body: JSON.stringify(in_json),
				headers: request.headers,
			},
		);
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
