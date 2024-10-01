import { attrTypeInfo } from "..";
import { Shell } from "lucide-react";
import { ReactElement } from "react";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";
import { useForm, UseFormReturnType } from "@mantine/form";
import {
	AttrCommonOptions,
	AttrNameEntry,
	AttrSubmitButtons,
} from "../_basicform";
import { Select } from "@mantine/core";

export const _hashAttrType: attrTypeInfo = {
	pretty_name: "Hash",
	serialize_as: "Hash",
	icon: <Shell />,
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
		hash_type: components["schemas"]["HashType"] | null;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
			is_not_null: false,
			hash_type: null,
		},
		validate: {
			new_attr_name: (value) =>
				value === null
					? "Attribute name is required"
					: value.trim().length === 0
					? "Attribute name cannot be empty"
					: null,

			hash_type: (value) =>
				value === null
					? "Hash type is required"
					: value.trim().length === 0
					? "Hash type cannot be empty"
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
				if (values.new_attr_name === null || values.hash_type === null) {
					// This is unreachable
					reset();
					return;
				}

				doCreate.mutate({
					data_type: {
						type: "Hash",
						hash_type: values.hash_type,
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
					disabled={doCreate.isPending}
					key={form.key("hash_type")}
					{...form.getInputProps("hash_type")}
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
