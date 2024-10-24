import { ReactElement } from "react";
import { useForm, UseFormReturnType } from "@mantine/form";
import { Button, Switch, Text, TextInput } from "@mantine/core";
import { components } from "../api/openapi";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "../api/client";

export function BasicForm(params: {
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

				<AttrCommonOptions
					form={form as UseFormReturnType<unknown>}
					isLoading={doCreate.isPending}
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

export function AttrNameEntry(params: {
	form: UseFormReturnType<unknown>;
	isLoading: boolean;
}) {
	return (
		<TextInput
			data-autofocus
			placeholder="New attribute name"
			disabled={params.isLoading}
			key={params.form.key("new_attr_name")}
			{...params.form.getInputProps("new_attr_name")}
		/>
	);
}

export function AttrCommonOptions(params: {
	form: UseFormReturnType<unknown>;
	isLoading: boolean;
}) {
	return (
		<>
			<Switch
				label="Unique"
				key={params.form.key("is_unique")}
				disabled={params.isLoading}
				{...params.form.getInputProps("is_unique")}
			/>

			<Switch
				label="Not null"
				key={params.form.key("is_not_null")}
				disabled={params.isLoading}
				{...params.form.getInputProps("is_not_null")}
			/>
		</>
	);
}

export function AttrSubmitButtons(params: {
	form: UseFormReturnType<unknown>;
	errorMessage: string | null;
	isLoading: boolean;
	reset: () => void;
}) {
	return (
		<>
			<Button.Group style={{ marginTop: "1rem" }}>
				<Button
					variant="light"
					fullWidth
					color="red"
					onClick={params.reset}
					disabled={params.isLoading}
				>
					Cancel
				</Button>
				<Button
					variant="filled"
					fullWidth
					color={params.errorMessage === null ? "green" : "red"}
					loading={params.isLoading}
					type="submit"
				>
					Create attribute
				</Button>
			</Button.Group>

			{params.errorMessage ? (
				<Text c="red" ta="center">
					{params.errorMessage}
				</Text>
			) : null}
		</>
	);
}
