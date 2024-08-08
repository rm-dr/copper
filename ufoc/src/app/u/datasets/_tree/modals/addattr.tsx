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
import { ModalBase } from "@/app/components/modal_base";
import { attrTypes } from "@/app/_util/attrs";
import { IconPlus } from "@tabler/icons-react";
import { XIcon } from "@/app/components/icons";
import { APIclient } from "@/app/_util/api";
import { components, paths } from "@/app/_util/api/openapi";

// TODO: make this a form

export function useAddAttrModal(params: {
	dataset_name: string;
	class: components["schemas"]["ClassInfo"];
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
	const [newAttrType, setNewAttrType] = useState<
		| paths["/attr/add"]["post"]["requestBody"]["content"]["application/json"]["data_type"]["type"]
		| null
	>(null);

	// This is an object set by an attributes's "extra params" node.
	// This is expanded directly into the new attr POST, see `add_attr` below.
	const [newAttrParams, setNewAttrParams] = useState<null | any>(null);

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

		let data_type: paths["/attr/add"]["post"]["requestBody"]["content"]["application/json"]["data_type"];
		if (newAttrType === "Hash") {
			data_type = {
				type: newAttrType,
				hash_type: newAttrParams.hash_type,
			};
		} else if (newAttrType === "Reference") {
			data_type = {
				type: newAttrType,
				class: newAttrParams.class,
			};
		} else {
			data_type = {
				type: newAttrType,
			};
		}

		APIclient.POST("/attr/add", {
			body: {
				class: params.class.handle,
				dataset: params.dataset_name,
				new_attr_name: newAttrName,
				data_type,
				options: {
					unique: false,
				},
			},
		}).then(({ data, error }) => {
			setLoading(false);

			if (error !== undefined) {
				setErrorMessage((m) => {
					return {
						...m,
						response: error,
					};
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
						<Text c="gray" span>{` ${params.class.name}`}</Text>:
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

				{NewAttrParamsInput === null ? null : (
					<div style={{ marginTop: "1rem", marginBottom: "1rem" }}>
						<NewAttrParamsInput
							dataset_name={params.dataset_name}
							errorMessage={errorMessage.extra_params}
							setErrorMessage={(m: any) => {
								setErrorMessage((e) => ({ ...e, extra_params: m }));
							}}
							onChange={(x: any) => {
								console.log(x);
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
						onClick={close}
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
						leftSection={<XIcon icon={IconPlus} />}
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
