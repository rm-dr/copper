import { Group, Select, SelectProps, Text } from "@mantine/core";
import { ReactElement, useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { ModalBase } from "@/app/components/modal_base";
import { attrTypes } from "@/app/_util/attrs";
import { components, paths } from "@/app/_util/api/openapi";

// TODO: make this a form

export function useAddAttrModal(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [errorMessage, setErrorMessage] = useState<{
		type: string | null;
		response: string | null;
	}>({ type: null, response: null });

	const [newAttrType, setNewAttrType] = useState<
		| paths["/attr/add"]["post"]["requestBody"]["content"]["application/json"]["data_type"]["type"]
		| null
	>(null);

	// Get input ui for attr-specific parameters
	let NewAttrForm:
		| null
		| ((params: {
				dataset_name: string;
				class: components["schemas"]["ClassInfo"];
				close: () => void;
		  }) => ReactElement) = null;

	if (newAttrType !== null) {
		const d = attrTypes.find((x) => {
			return x.serialize_as === newAttrType;
		});
		if (d !== undefined && d.params !== null) {
			// This is a function, but DON'T RUN IT!
			// It's a react component that is placed into tsx below.
			NewAttrForm = d.params.form;
		}
	}

	const renderSelectOption: SelectProps["renderOption"] = ({
		option,
		checked,
	}) => {
		let icon = null;
		const d = attrTypes.find((x) => {
			return x.serialize_as === option.value;
		});
		if (d !== undefined) {
			icon = d.icon;
		}
		return (
			<Group flex="1" gap="xs">
				<div
					style={{
						// center icon vertically
						display: "flex",
						flexDirection: "column",
						justifyContent: "center",
						alignItems: "center",
						height: "100%",
						// looks
						width: "1.5rem",
						color: "var(--mantine-color-dimmed)",
					}}
				>
					{icon}
				</div>

				{option.label}
			</Group>
		);
	};

	return {
		open,
		modal: (
			<ModalBase
				opened={opened}
				close={() => {
					// Reset everything on close
					setNewAttrType(null);
					setErrorMessage({
						type: null,
						response: null,
					});
					close();
				}}
				title="Add an attribute"
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add an attribute to the class
						<Text c="gray" span>{` ${params.class.name}`}</Text>:
					</Text>
				</div>
				<Select
					renderOption={renderSelectOption}
					required={true}
					style={{ marginTop: "1rem" }}
					placeholder={"select attr type"}
					data={attrTypes.map((x) => ({
						label: x.pretty_name,
						value: x.serialize_as,
						disabled: false,
					}))}
					error={errorMessage.type !== null}
					onChange={(val) => {
						setNewAttrType(
							val as paths["/attr/add"]["post"]["requestBody"]["content"]["application/json"]["data_type"]["type"],
						);
						setErrorMessage((m) => {
							return {
								...m,
								type: null,
							};
						});
					}}
					comboboxProps={{
						transitionProps: {
							transition: "fade-down",
							duration: 200,
						},
					}}
					clearable
				/>

				{NewAttrForm === null ? null : (
					<div style={{ marginTop: "0.5rem" }}>
						<NewAttrForm
							dataset_name={params.dataset_name}
							class={params.class}
							close={() => {
								params.onSuccess();
								close();
							}}
						/>
					</div>
				)}
			</ModalBase>
		),
	};
}
