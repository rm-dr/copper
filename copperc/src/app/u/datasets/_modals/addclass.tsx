import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";

export function useAddClassModal(params: {
	dataset_id: number;
	dataset_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			name: "",
		},
		validate: {
			name: (value) => {
				if (value.trim().length === 0) {
					return "This field is required";
				}

				return null;
			},
		},
	});

	const doCreate = useMutation({
		mutationFn: async (body: components["schemas"]["NewClassRequest"]) => {
			return await edgeclient.POST("/dataset/{dataset_id}/class", {
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
		doCreate.reset();
		form.reset();
		close();
	};

	return {
		open,
		modal: (
			<ModalBaseSmall
				opened={opened}
				close={reset}
				title="Add class"
				keepOpen={doCreate.isPending}
			>
				<form
					onSubmit={form.onSubmit((values) => {
						doCreate.mutate({ name: values.name });
					})}
				>
					<div className={modalStyle.modal_outer_container}>
						<div className={modalStyle.modal_input_container}>
							<Text c="dimmed" size="sm">
								Add a class to the dataset
								<Text
									c="var(--mantine-primary-color-4)"
									span
								>{` ${params.dataset_name}`}</Text>
								:
							</Text>

							<TextInput
								data-autofocus
								placeholder="Enter class name"
								disabled={doCreate.isPending}
								key={form.key("name")}
								{...form.getInputProps("name")}
							/>
						</div>

						<Button.Group style={{ width: "100%" }}>
							<Button
								variant="light"
								fullWidth
								c="primary"
								onClick={reset}
								disabled={doCreate.isPending}
							>
								Cancel
							</Button>
							<Button
								variant="filled"
								fullWidth
								c="primary"
								loading={doCreate.isPending}
								type="submit"
							>
								Confirm
							</Button>
						</Button.Group>

						{doCreate.error ? (
							<Text c="red" ta="center">
								{doCreate.error.message}
							</Text>
						) : null}
					</div>
				</form>
			</ModalBaseSmall>
		),
	};
}
