import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";

export function useDeleteDatasetModal(params: {
	dataset_id: number;
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			dataset_name: "",
		},
		validate: {
			dataset_name: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				if (value !== params.dataset_name) {
					return "Dataset name doesn't match";
				}

				return null;
			},
		},
	});

	const doDelete = useMutation({
		mutationFn: async () => {
			return await edgeclient.DELETE("/dataset/{dataset_id}", {
				params: { path: { dataset_id: params.dataset_id } },
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

	const reset = () => {
		doDelete.reset();
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
				keepOpen={doDelete.isPending}
			>
				<form
					onSubmit={form.onSubmit(() => {
						doDelete.mutate();
					})}
				>
					<div className={modalStyle.modal_outer_container}>
						<div className={modalStyle.modal_input_container}>
							<div>
								<Text c="red" size="sm">
									This action will irreversably destroy data.
								</Text>
								<Text c="red" size="sm">
									Enter
									<Text c="orange" span>{` ${params.dataset_name} `}</Text>
									below to confirm.
								</Text>
							</div>

							<TextInput
								data-autofocus
								placeholder="Enter dataset name"
								disabled={doDelete.isPending}
								key={form.key("dataset_name")}
								{...form.getInputProps("dataset_name")}
							/>
						</div>

						<Button.Group style={{ width: "100%" }}>
							<Button
								variant="light"
								fullWidth
								color="red"
								onClick={reset}
								disabled={doDelete.isPending}
							>
								Cancel
							</Button>
							<Button
								variant="filled"
								fullWidth
								color="red"
								loading={doDelete.isPending}
								type="submit"
							>
								Confirm
							</Button>
						</Button.Group>

						{doDelete.error ? (
							<Text c="red" ta="center">
								{doDelete.error.message}
							</Text>
						) : null}
					</div>
				</form>
			</ModalBaseSmall>
		),
	};
}
