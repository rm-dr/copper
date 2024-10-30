import { ReactElement } from "react";
import { attrTypeInfo } from "..";
import { Binary } from "lucide-react";
import { useMutation } from "@tanstack/react-query";
import { components } from "@/lib/api/openapi";
import { edgeclient } from "@/lib/api/client";
import { useForm, UseFormReturnType } from "@mantine/form";
import { AttrNameEntry, AttrSubmitButtons } from "../_basicform";
import { Switch } from "@mantine/core";

export const _blobAttrType: attrTypeInfo<"Blob"> = {
	pretty_name: "Blob",
	serialize_as: "Blob",
	icon: <Binary />,
	create_params: {
		form: (params) => BlobForm({ attr_type: { type: "Blob" }, ...params }),
	},

	table_cell: () => {
		return "Blob";
	},
};

// Same as basicform, but with the "unique" switch hidden.
// It has no effect on blobs.
function BlobForm(params: {
	attr_type: { type: "Text" | "Boolean" | "Blob" };
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
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
			is_not_null: false,
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
					data_type: params.attr_type,
					name: values.new_attr_name,
					options: {
						is_unique: values.is_unique,
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

				<Switch
					label="Not null"
					key={form.key("is_not_null")}
					disabled={doCreate.isPending}
					{...form.getInputProps("is_not_null")}
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
