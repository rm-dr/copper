import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";

export function useRenameAttributeModal(params: {
	attribute_id: number;
	attribute_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			new_name: params.attribute_name,
		},
		validate: {
			new_name: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				return null;
			},
		},
	});

	const doRename = useMutation({
		mutationFn: async (
			body: components["schemas"]["RenameAttributeRequest"],
		) => {
			return await edgeclient.PATCH("/attribute/{attribute_id}", {
				params: { path: { attribute_id: params.attribute_id } },
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
		doRename.reset();
		form.reset();
		close();
	};

	return {
		open,
		modal: (
			<ModalBaseSmall
				opened={opened}
				close={reset}
				title="Rename attribute"
				keepOpen={doRename.isPending}
			>
				<form
					onSubmit={form.onSubmit((values) => {
						doRename.mutate({ new_name: values.new_name });
					})}
				>
					<div className={modalStyle.modal_outer_container}>
						<div className={modalStyle.modal_input_container}>
							<Text c="dimmed" size="sm">
								You are renaming the attribute
								<Text
									c="var(--mantine-primary-color-4)"
									span
								>{` ${params.attribute_name}`}</Text>
								.
							</Text>

							<TextInput
								data-autofocus
								placeholder="Enter attribute name"
								disabled={doRename.isPending}
								key={form.key("new_name")}
								{...form.getInputProps("new_name")}
							/>
						</div>

						<Button.Group style={{ width: "100%" }}>
							<Button
								variant="light"
								fullWidth
								c="primary"
								onClick={reset}
								disabled={doRename.isPending}
							>
								Cancel
							</Button>
							<Button
								variant="filled"
								fullWidth
								c="primary"
								loading={doRename.isPending}
								type="submit"
							>
								Confirm
							</Button>
						</Button.Group>

						{doRename.error ? (
							<Text c="red" ta="center">
								{doRename.error.message}
							</Text>
						) : null}
					</div>
				</form>
			</ModalBaseSmall>
		),
	};
}
