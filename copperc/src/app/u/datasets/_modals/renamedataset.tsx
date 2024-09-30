import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";

export function useRenameDatasetModal(params: {
	dataset_id: number;
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			new_name: params.dataset_name,
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
		mutationFn: async (body: components["schemas"]["RenameDatasetRequest"]) => {
			return await edgeclient.PATCH("/dataset/{dataset_id}", {
				params: { path: { dataset_id: params.dataset_id } },
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
				title="Delete dataset"
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
								You are renaming the dataset
								<Text
									c="var(--mantine-primary-color-4)"
									span
								>{` ${params.dataset_name}`}</Text>
								.
							</Text>

							<TextInput
								data-autofocus
								placeholder="Enter dataset name"
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
