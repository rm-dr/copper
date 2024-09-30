import { Select, Text } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { ModalBaseSmall } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";
import { ReactElement, useState } from "react";
import { attrTypes } from "@/lib/attributes";

export function useAddAttributeModal(params: {
	dataset_id: number;
	class_id: number;
	class_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const [errorMessage, setErrorMessage] = useState<{
		type: string | null;
		response: string | null;
	}>({ type: null, response: null });

	const [newAttrType, setNewAttrType] = useState<
		components["schemas"]["AttrDataStub"]["type"] | null
	>(null);

	// Get input ui for attr-specific parameters
	let NewAttrForm:
		| null
		| ((params: {
				dataset_id: number;
				class_id: number;
				onSuccess: () => void;
				close: () => void;
		  }) => ReactElement) = null;

	if (newAttrType !== null) {
		const d = attrTypes.find((x) => {
			return x.serialize_as === newAttrType;
		});
		if (d !== undefined && d.create_params !== null) {
			// This is a function, but don't call it here!
			// It's a react component that is placed into tsx below.
			NewAttrForm = d.create_params.form;
		}
	}

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
			}

			throw new Error(res.error);
		},

		onError: (err) => {
			throw err;
		},
	});

	const reset = () => {
		doCreate.reset();
		setNewAttrType(null);
		setErrorMessage({
			type: null,
			response: null,
		});
		close();
	};

	return {
		open,
		modal: (
			<ModalBaseSmall
				opened={opened}
				close={reset}
				title="Add an attribute"
				keepOpen={doCreate.isPending}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add an attribute to the class
						<Text
							c="var(--mantine-primary-color-4)"
							span
						>{` ${params.class_name}`}</Text>
						:
					</Text>
				</div>

				<Select
					required={true}
					placeholder={"select attribute type"}
					data={attrTypes.map((x) => ({
						label: x.pretty_name,
						value: x.serialize_as,
						disabled: false,
					}))}
					disabled={doCreate.isPending}
					error={errorMessage.type !== null}
					onChange={(val) => {
						setNewAttrType(
							val as components["schemas"]["AttrDataStub"]["type"],
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
							dataset_id={params.dataset_id}
							class_id={params.class_id}
							onSuccess={() => {
								params.onSuccess();
								close();
							}}
							close={() => {
								close();
							}}
						/>
					</div>
				)}

				{doCreate.error ? (
					<Text c="red" ta="center">
						{doCreate.error.message}
					</Text>
				) : null}
			</ModalBaseSmall>
		),
	};
}
