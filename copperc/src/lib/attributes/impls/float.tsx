import { attrTypeInfo } from "..";
import { Hash } from "lucide-react";
import {
	AttrCommonOptions,
	AttrNameEntry,
	AttrSubmitButtons,
} from "../_basicform";
import { ReactElement } from "react";
import { useMutation } from "@tanstack/react-query";
import { useForm, UseFormReturnType } from "@mantine/form";
import { Switch } from "@mantine/core";
import { components } from "@/lib/api/openapi";
import { edgeclient } from "@/lib/api/client";

export const _floatAttrType: attrTypeInfo = {
	pretty_name: "Float",
	serialize_as: "Float",
	icon: <Hash />,
	create_params: {
		form: (params) => Form(params),
	},
};

function Form(params: {
	class_id: number;
	onSuccess: () => void;
	close: () => void;
}): ReactElement {
	const doCreate = useMutation({
		mutationFn: async (body: components["schemas"]["NewAttributeRequest"]) => {
			return await edgeclient.POST("/class/{class_id}/attribute", {
				params: { path: { class_id: params.class_id } },
				body,
			});
		},

		onSuccess: async (res) => {
			if (res.response.status === 200) {
				reset();
				params.onSuccess();
			} else {
				throw new Error(res.error);
			}
		},

		onError: (err) => {
			throw err;
		},
	});

	const form = useForm<{
		new_attr_name: string | null;
		is_unique: boolean;
		is_not_null: boolean;
		is_non_negative: boolean;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
			is_not_null: false,
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
		params.close();
	};

	return (
		<form
			onSubmit={form.onSubmit((values) => {
				if (values.new_attr_name === null) {
					// This is unreachable
					reset();
					return;
				}

				doCreate.mutate({
					data_type: {
						type: "Float",
						is_non_negative: values.is_non_negative,
					},
					name: values.new_attr_name,
					options: {
						unique: values.is_unique,
						is_not_null: values.is_not_null,
					},
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
				<AttrNameEntry
					form={form as UseFormReturnType<unknown>}
					isLoading={doCreate.isPending}
				/>

				<AttrCommonOptions
					form={form as UseFormReturnType<unknown>}
					isLoading={doCreate.isPending}
				/>

				<Switch
					label="Non-negative"
					key={form.key("is_non_negative")}
					{...form.getInputProps("is_non_negative")}
				/>

				<AttrSubmitButtons
					form={form as UseFormReturnType<unknown>}
					errorMessage={doCreate.error === null ? null : doCreate.error.message}
					isLoading={doCreate.isPending}
					reset={reset}
				/>
			</div>
		</form>
	);
}