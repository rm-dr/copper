import { ReactElement, useState } from "react";
import { components } from "../../api/openapi";
import { useForm, UseFormReturnType } from "@mantine/form";
import { APIclient } from "../../api";
import { Button, Switch, Text, TextInput } from "@mantine/core";
import { IconPlus } from "@tabler/icons-react";
import { XIcon } from "../../../components/icons";

export function BaseForm(params: {
	attr_type: Exclude<
		components["schemas"]["MetastoreDataStub"],
		{ type: "Reference" | "Hash" | "Integer" | "Float" }
	>;
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	close: () => void;
}): ReactElement {
	const [isLoading, setLoading] = useState(false);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const form = useForm<{
		new_attr_name: string | null;
		is_unique: boolean;
	}>({
		mode: "uncontrolled",
		initialValues: {
			new_attr_name: null,
			is_unique: false,
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
						data_type: params.attr_type,
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

export function AttrNameEntry(params: {
	form: UseFormReturnType<any>;
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
	form: UseFormReturnType<any>;
	isLoading: boolean;
}) {
	return (
		<>
			<Switch
				label="Unique"
				key={params.form.key("is_unique")}
				{...params.form.getInputProps("is_unique")}
			/>
		</>
	);
}

export function AttrSubmitButtons(params: {
	form: UseFormReturnType<any>;
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
					leftSection={<XIcon icon={IconPlus} />}
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
