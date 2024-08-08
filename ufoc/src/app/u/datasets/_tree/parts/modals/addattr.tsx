import { XIconPlus } from "@/app/components/icons";
import {
	Button,
	Group,
	Select,
	SelectProps,
	Text,
	TextInput,
} from "@mantine/core";
import { useState } from "react";
import { useDisclosure } from "@mantine/hooks";
import { ModalBase } from "./modal_base";
import { attrTypes } from "../../attrs";

// TODO: make this a form

export function useAddAttrModal(params: {
	dataset_name: string;
	class_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);
	const [isLoading, setLoading] = useState(false);

	const [errorMessage, setErrorMessage] = useState<{
		name: string | null;
		type: string | null;
		response: string | null;
		extra_params: null | any;
	}>({ name: null, type: null, response: null, extra_params: null });

	const [newAttrName, setNewAttrName] = useState("");
	const [newAttrType, setNewAttrType] = useState<string | null>(null);

	// This is an object set by an attributes's "extra params" node.
	// This is expanded directly into the new attr POST, see `add_attr` below.
	const [newAttrParams, setNewAttrParams] = useState<null | Object>(null);

	// Get input ui for attr-specific parameters
	let NewAttrParamsInput: null | any = null;
	let newAttrExtraParams: null | any = null;
	if (newAttrType !== null) {
		const d = attrTypes.find((x) => {
			return x.serialize_as === newAttrType;
		});
		if (d !== undefined && d.extra_params !== null) {
			// This is a function, but DON'T RUN IT!
			// It's a react component that is placed into tsx below.
			NewAttrParamsInput = d.extra_params.node;
			newAttrExtraParams = d.extra_params;
		}
	}

	const add_attr = () => {
		setLoading(true);
		if (newAttrName == "") {
			setLoading(false);
			setErrorMessage((m) => {
				return {
					...m,
					name: "Name cannot be empty",
				};
			});
			return;
		} else if (newAttrType === null) {
			setLoading(false);
			setErrorMessage((m) => {
				return {
					...m,
					type: "Type cannot be empty",
				};
			});
			return;
		} else if (newAttrExtraParams !== null) {
			if (
				!newAttrExtraParams.inputs_ok({
					state: newAttrParams,
					setErrorMessage: (m: any) => {
						setErrorMessage((e) => ({ ...e, extra_params: m }));
					},
				})
			) {
				setLoading(false);
				return;
			}
		}

		setErrorMessage({
			name: null,
			type: null,
			response: null,
			extra_params: null,
		});

		let extra_params = {};
		if (newAttrParams !== null) {
			extra_params = newAttrParams;
		}

		fetch("/api/attr/add", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				class: params.class_name,
				dataset: params.dataset_name,
				attr: newAttrName,
				data_type: {
					type: newAttrType,
					...extra_params,
				},
				options: {
					unique: false,
				},
			}),
		}).then((res) => {
			setLoading(false);
			if (res.status == 400) {
				res.text().then((text) => {
					setErrorMessage((m) => {
						return {
							...m,
							response: text,
						};
					});
				});
			} else if (!res.ok) {
				res.text().then((text) => {
					setErrorMessage((m) => {
						return {
							...m,
							response: `Error ${res.status}: ${text}`,
						};
					});
				});
			} else {
				params.onSuccess();
				setLoading(false);
				close();
			}
		});
	};

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
					setNewAttrName("");
					setNewAttrType(null);
					setNewAttrParams(null);
					setErrorMessage({
						name: null,
						type: null,
						response: null,
						extra_params: null,
					});
					close();
				}}
				title="Add an attribute"
				keepOpen={isLoading}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add an attribute to the class
						<Text c="gray" span>{` ${params.class_name}`}</Text>:
					</Text>
				</div>
				<TextInput
					data-autofocus
					placeholder="New attr name"
					disabled={isLoading}
					error={errorMessage.name !== null}
					onChange={(e) => {
						setNewAttrName(e.currentTarget.value);
						setErrorMessage((m) => {
							return {
								...m,
								name: null,
							};
						});
					}}
				/>

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
						setNewAttrType(val);
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

				{NewAttrParamsInput === null ? null : (
					<div style={{ marginTop: "1rem", marginBottom: "1rem" }}>
						<NewAttrParamsInput
							dataset_name={params.dataset_name}
							errorMessage={errorMessage.extra_params}
							setErrorMessage={(m: any) => {
								setErrorMessage((e) => ({ ...e, extra_params: m }));
							}}
							onChange={(x: any) => {
								setNewAttrParams(x);
							}}
						/>
					</div>
				)}

				<Button.Group style={{ marginTop: "1rem" }}>
					<Button
						variant="light"
						fullWidth
						color="red"
						onMouseDown={close}
						disabled={isLoading}
					>
						Cancel
					</Button>
					<Button
						variant="filled"
						color={
							Object.values(errorMessage).every((x) => x === null)
								? "green"
								: "red"
						}
						fullWidth
						leftSection={<XIconPlus />}
						onClick={add_attr}
					>
						Create Attribute
					</Button>
				</Button.Group>

				<Text c="red" ta="center">
					{/* TODO: this is ugly */}
					{errorMessage.response
						? errorMessage.response
						: errorMessage.name
						? errorMessage.name
						: errorMessage.type
						? errorMessage.type
						: errorMessage.extra_params
						? errorMessage.extra_params
						: ""}
				</Text>
			</ModalBase>
		),
	};
}
