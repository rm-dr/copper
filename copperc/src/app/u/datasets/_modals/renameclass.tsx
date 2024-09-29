import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";

export function useRenameClassModal(params: {
	class_id: number;
	class_name: string;
	onSuccess: () => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			new_name: params.class_name,
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
		mutationFn: async (body: components["schemas"]["RenameClassRequest"]) => {
			return await edgeclient.PATCH("/class/{class_id}", {
				params: { path: { class_id: params.class_id } },
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
				title="Rename class"
				keepOpen={doRename.isPending}
			>
				<div
					style={{
						marginBottom: "1rem",
					}}
				>
					<Text c="dimmed" size="sm">
						You are renaming the class
						<Text
							c="var(--mantine-primary-color-4)"
							span
						>{` ${params.class_name}`}</Text>
						.
					</Text>
				</div>

				<form
					onSubmit={form.onSubmit((values) => {
						doRename.mutate({ new_name: values.new_name });
					})}
				>
					<TextInput
						data-autofocus
						placeholder="Enter class name"
						disabled={doRename.isPending}
						key={form.key("new_name")}
						{...form.getInputProps("new_name")}
					/>

					<Button.Group style={{ marginTop: "1rem" }}>
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
				</form>
			</ModalBaseSmall>
		),
	};
}
