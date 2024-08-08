import { Select, Text, Textarea } from "@mantine/core";
import { attrTypeInfo } from ".";
import { IconAnalyze } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";

export const _hashAttrType: attrTypeInfo = {
	pretty_name: "Hash",
	serialize_as: "Hash",
	icon: <XIcon icon={IconAnalyze} />,
	extra_params: {
		inputs_ok: checkHash,
		node: HashParams,
	},

	value_preview: (params) => {
		if (params.attr.value === null) {
			return (
				<>
					<Text c="dimmed" span>{`${params.attr.hash_type}: `}</Text>
					<Text c="dimmed" fs="italic" span>
						no value
					</Text>
				</>
			);
		} else {
			return (
				<>
					<Text c="dimmed" span>{`${params.attr.hash_type}: `}</Text>
					<Text ff="monospace" span>
						{params.attr.value}
					</Text>
				</>
			);
		}
	},

	editor: {
		type: "inline",

		old_value: (params) => {
			if (params.attr.value == null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return params.attr.value;
			}
		},

		new_value: (params) => {
			return (
				<Textarea
					radius="0px"
					placeholder="no value"
					autosize
					minRows={1}
					defaultValue={params.attr.value}
					onChange={params.onChange}
				/>
			);
		},
	},
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
