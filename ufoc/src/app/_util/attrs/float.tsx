import { IconDecimal, IconPlus } from "@tabler/icons-react";
import { attrTypeInfo } from ".";
import { Button, NumberInput, Switch, Text, TextInput } from "@mantine/core";
import { XIcon } from "@/app/components/icons";
import { APIclient } from "../api";
import { useForm } from "@mantine/form";
import { ReactElement, useState } from "react";
import { components } from "../api/openapi";
import {
	AttrCommonOptions,
	AttrNameEntry,
	AttrSubmitButtons,
} from "./helpers/baseform";

export const _floatAttrType: attrTypeInfo = {
	pretty_name: "Float",
	serialize_as: "Float",
	icon: <XIcon icon={IconDecimal} />,
	params: {
		form: FloatForm,
	},

	value_preview: (params) => {
		if (params.attr_value.type !== "Float") {
			return <>Unreachable!</>;
		}

		if (params.attr_value.value === null) {
			return (
				<Text c="dimmed" fs="italic">
					no value
				</Text>
			);
		} else {
			return <Text>{params.attr_value.value}</Text>;
		}
	},

	editor: {
		type: "inline",
		old_value: (params) => {
			if (params.attr_value.type !== "Float") {
				return <>Unreachable!</>;
			}

			if (params.attr_value.value === null) {
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
			if (params.attr_value.type !== "Float") {
				return <>Unreachable!</>;
			}

			return (
				<NumberInput
					placeholder="empty value"
					allowDecimal={true}
					defaultValue={params.attr_value.value || undefined}
				/>
			);
		},
	},
};

export function FloatForm(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	close: () => void;
}): ReactElement {
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		new_attr_name: string | null;
		is_non_negative: boolean;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_non_negative: false,
		},
		validate: {
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

				if (values.new_attr_name === null) {
					// This is unreachable
					params.close();
					return;
				}

				APIclient.POST("/attr/add", {
					params: {},
					body: {
						class: params.class.handle,
						dataset: params.dataset_name,
						new_attr_name: values.new_attr_name,
						data_type: {
							type: "Float",
							is_non_negative: values.is_non_negative,
						},
						options: {
							unique: false,
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

				<Switch
					label="Non-negative"
					key={form.key("is_non_negative")}
					{...form.getInputProps("is_non_negative")}
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
