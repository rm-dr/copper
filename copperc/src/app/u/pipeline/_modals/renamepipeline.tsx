import { Button, Text, TextInput } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { useForm } from "@mantine/form";
import { ModalBaseSmall, modalStyle } from "@/components/modalbase";
import { useMutation } from "@tanstack/react-query";
import { edgeclient } from "@/lib/api/client";
import { components } from "@/lib/api/openapi";

export function useRenamePipelineModal(params: {
	pipeline_id: number;
	pipeline_name: string;
	onSuccess: (new_info: components["schemas"]["PipelineInfo"]) => void;
}) {
	const [opened, { open, close }] = useDisclosure(false);

	const form = useForm({
		mode: "uncontrolled",
		initialValues: {
			new_name: params.pipeline_name,
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
		mutationFn: async (new_name: string) => {
			return await edgeclient.PATCH("/pipeline/{pipeline_id}", {
				params: { path: { pipeline_id: params.pipeline_id } },
				body: { new_name },
			});
		},

		onSuccess: async (res) => {
			if (res.response.status === 200) {
				reset();
				params.onSuccess(res.data!);
			} else {
				throw new Error(res.error);
			}
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
				title="Rename pipeline"
				keepOpen={doRename.isPending}
			>
				<form
					onSubmit={form.onSubmit((values) => {
						doRename.mutate(values.new_name);
					})}
				>
					<div className={modalStyle.modal_outer_container}>
						<div className={modalStyle.modal_input_container}>
							<Text c="dimmed" size="sm">
								You are renaming the pipeline
								<Text
									c="var(--mantine-primary-color-4)"
									span
								>{` ${params.pipeline_name}`}</Text>
								.
							</Text>

							<TextInput
								data-autofocus
								placeholder="Enter pipeline name"
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
