import { Button, Select, Text, Textarea, TextInput } from "@mantine/core";
import { attrTypeInfo } from ".";
import { IconAnalyze, IconPlus } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { useForm } from "@mantine/form";
import { ReactElement, useState } from "react";
import { APIclient } from "../api";
import { components } from "../api/openapi";
import {
	AttrCommonOptions,
	AttrNameEntry,
	AttrSubmitButtons,
} from "./helpers/baseform";

export const _hashAttrType: attrTypeInfo = {
	pretty_name: "Hash",
	serialize_as: "Hash",
	icon: <XIcon icon={IconAnalyze} />,
	params: {
		form: HashForm,
	},

	value_preview: (params) => {
		if (params.attr_value.type !== "Hash") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.value === null) {
			return (
				<>
					<Text c="dimmed" span>{`${params.attr_value.hash_type}: `}</Text>
					<Text c="dimmed" fs="italic" span>
						no value
					</Text>
				</>
			);
		} else {
			return (
				<>
					<Text c="dimmed" span>{`${params.attr_value.hash_type}: `}</Text>
					<Text ff="monospace" span>
						{params.attr_value.value}
					</Text>
				</>
			);
		}
	},

	editor: {
		type: "inline",

		old_value: (params) => {
			if (params.attr_value.type !== "Hash") {
				return <>Unreachable!</>;
			}

			if (params.attr_value.value == null) {
				return (
					<Text c="dimmed" fs="italic">
						no value
					</Text>
				);
			} else {
				return <Text>{params.attr_value.value}</Text>;
			}
		},

		new_value: (params) => {
			if (params.attr_value.type !== "Hash") {
				return <>Unreachable!</>;
			}

			return (
				<Textarea
					radius="0px"
					placeholder="no value"
					autosize
					minRows={1}
					defaultValue={params.attr_value.value || undefined}
					onChange={params.onChange}
				/>
			);
		},
	},
};

function HashForm(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	close: () => void;
}): ReactElement {
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		hash_type: components["schemas"]["HashType"] | null;
		new_attr_name: string | null;
		is_unique: boolean;
	}>({
		mode: "uncontrolled",
		initialValues: {
			hash_type: null,
			new_attr_name: null,
			is_unique: false,
		},
		validate: {
			hash_type: (value) =>
				value === null
					? "Hash type is required"
					: value.trim().length === 0
					? "Hash type cannot be empty"
					: null,
			new_attr_name: (value) =>
				value === null
					? "Attribute name is required"
					: value.trim().length === 0
					? "Attribute name cannot be empty"
					: null,
		},
	});

	const reset = () => {
		form.reset();
		setLoading(false);
		setErrorMessage(null);
		params.close();
	};

	return (
		<form
			onSubmit={form.onSubmit((values) => {
				setLoading(true);
				setErrorMessage(null);

				if (values.hash_type === null || values.new_attr_name === null) {
					// This is unreachable
					params.close();
					return;
				}

				APIclient.POST("/attr/add", {
					body: {
						class: params.class.handle,
						dataset: params.dataset_name,
						new_attr_name: values.new_attr_name,
						data_type: {
							type: "Hash",
							hash_type: values.hash_type,
						},
						options: {
							unique: values.is_unique,
						},
					},
				}).then(({ data, error }) => {
					setLoading(false);
					if (error !== undefined) {
						setErrorMessage(error);
					} else {
						setLoading(false);
						params.close();
					}
				});
			})}
		>
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					gap: "0.5rem",
				}}
			>
				<AttrNameEntry form={form} isLoading={isLoading} />

				<Select
					required={true}
					placeholder={"Hash type"}
					data={[
						// Hash types the server supports
						{ label: "MD5", value: "MD5", disabled: false },
						{ label: "SHA256", value: "SHA256", disabled: false },
						{ label: "SHA512", value: "SHA512", disabled: false },
					]}
					clearable
					disabled={isLoading}
					key={form.key("hash_type")}
					{...form.getInputProps("hash_type")}
				/>

				<AttrCommonOptions form={form} isLoading={isLoading} />

				<AttrSubmitButtons
					form={form}
					errorMessage={errorMessage}
					isLoading={isLoading}
					reset={reset}
				/>
			</div>
		</form>
	);
}
