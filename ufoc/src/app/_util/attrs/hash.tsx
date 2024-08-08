import { Code, Select, Text } from "@mantine/core";
import { attrTypeInfo } from ".";
import { XIconAttrHash } from "@/app/components/icons";

export const _hashAttrType: attrTypeInfo = {
	pretty_name: "Hash",
	serialize_as: "Hash",
	icon: <XIconAttrHash />,
	extra_params: {
		inputs_ok: checkHash,
		node: HashParams,
	},

	value_preview: (params) => (
		<Text>
			{`${params.attr.hash_type} hash: `}
			<Code>{params.attr.value}</Code>
		</Text>
	),
};

function checkHash(params: {
	state: any;
	setErrorMessage: (message: null | any) => void;
}): boolean {
	if (params.state === null) {
		params.setErrorMessage("Hash type is required");
		return false;
	} else if (params.state.hash_type === null) {
		params.setErrorMessage("Hash type is required");
		return false;
	}

	return true;
}

function HashParams(params: {
	onChange: (state: null | any) => void;
	setErrorMessage: (message: null | any) => void;
	errorMessage: null | any;
}) {
	return (
		<Select
			required={true}
			placeholder={"select hash type"}
			data={[
				// Hash types the server supports
				{ label: "MD5", value: "MD5", disabled: false },
				{ label: "SHA256", value: "SHA256", disabled: false },
				{ label: "SHA512", value: "SHA512", disabled: false },
			]}
			clearable
			error={params.errorMessage !== null}
			onChange={(v) => {
				params.setErrorMessage(null);
				if (v == null) {
					params.onChange({ hash_type: null });
				} else {
					params.onChange({ hash_type: v });
				}
			}}
		/>
	);
}
