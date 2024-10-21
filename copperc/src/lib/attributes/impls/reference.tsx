import { attrTypeInfo } from "..";
import { Ampersand } from "lucide-react";
import {
	AttrCommonOptions,
	AttrNameEntry,
	AttrSubmitButtons,
} from "../_basicform";
import { useForm, UseFormReturnType } from "@mantine/form";
import { components } from "@/lib/api/openapi";
import { edgeclient } from "@/lib/api/client";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ReactElement } from "react";
import { Select } from "@mantine/core";

export const _referenceAttrType: attrTypeInfo = {
	pretty_name: "Reference",
	serialize_as: "Reference",
	icon: <Ampersand />,
	create_params: {
		form: (params) => Form(params),
	},
};

function Form(params: {
	dataset_id: number;
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

	const datasetinfo = useQuery({
		queryKey: ["dataset"],
		queryFn: async () => {
			const res = await edgeclient.GET("/dataset/{dataset_id}", {
				params: { path: { dataset_id: params.dataset_id } },
			});
			if (res.response.status === 401) {
				location.replace("/");
			}

			return res.data;
		},
	});

	const form = useForm<{
		new_attr_name: string | null;
		is_unique: boolean;
		is_not_null: boolean;
		reference_target_class: string | null;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
			is_not_null: false,
			reference_target_class: null,
		},
		validate: {
			new_attr_name: (value) =>
				value === null
					? "Attribute name is required"
					: value.trim().length === 0
					? "Attribute name cannot be empty"
					: null,

			reference_target_class: (value) =>
				value === null ? "Reference target is required" : null,
		},
	});

	const reset = () => {
		form.reset();
		params.close();
	};

	return (
		<form
			onSubmit={form.onSubmit((values) => {
				if (
					values.new_attr_name === null ||
					values.reference_target_class === null
				) {
					// This is unreachable
					reset();
					return;
				}

				// Parse value as an integer.
				// This should never fail, see select options.
				// Mantine select only supports string values.
				let c = null;
				try {
					c = parseInt(values.reference_target_class);
				} catch {}

				if (c === null) {
					return;
				}

				doCreate.mutate({
					data_type: {
						type: "Reference",
						class: c,
					},
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

				<AttrCommonOptions
					form={form as UseFormReturnType<unknown>}
					isLoading={doCreate.isPending}
				/>

				<Select
					required={true}
					placeholder={"Select class"}
					data={
						datasetinfo.data === undefined
							? []
							: datasetinfo.data.classes.map((x) => ({
									label: x.name,
									value: x.id.toString(),
							  }))
					}
					clearable
					disabled={doCreate.isPending}
					key={form.key("reference_target_class")}
					{...form.getInputProps("reference_target_class")}
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
