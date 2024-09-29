import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall } from "@/components/modalbase";
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

		onSuccess: async ({ response }) => {
			if (response === null) {
				return;
			}

			if (response.status !== 200) {
				throw new Error(await response.json());
			} else {
				reset();
				params.onSuccess();
			}
		},
		onError: (err) => {
			throw err;
		},
	});

	const reset = () => {
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
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						Add a class to the dataset
						<Text
							c="var(--mantine-primary-color-4)"
							span
						>{` ${params.dataset_name}`}</Text>
						:
					</Text>
				</div>

				<form
					onSubmit={form.onSubmit((values) => {
						doCreate.mutate({ name: values.name });
					})}
				>
					<TextInput
						data-autofocus
						placeholder="Enter class name"
						disabled={doCreate.isPending}
						key={form.key("name")}
						{...form.getInputProps("name")}
					/>

					<Button.Group style={{ marginTop: "1rem" }}>
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
				</form>
			</ModalBaseSmall>
		),
	};
}
