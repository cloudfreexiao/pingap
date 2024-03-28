import useConfigStore from "../states/config";
import { useParams } from "react-router-dom";

import Loading from "../components/loading";
import FormEditor, {
  FormItem,
  FormItemCategory,
} from "../components/form-editor";

export default function UpstreamInfo() {
  const [initialized, config, update] = useConfigStore((state) => [
    state.initialized,
    state.data,
    state.update,
  ]);
  const { name } = useParams();
  if (!initialized) {
    return <Loading />;
  }
  const upstreams = config.upstreams || {};
  const upstream = upstreams[name || ""];

  const arr: FormItem[] = [
    {
      id: "addrs",
      label: "Upstream Addrs",
      defaultValue: upstream.addrs,
      span: 12,
      category: FormItemCategory.ADDRS,
    },
    {
      id: "algo",
      label: "Load balancer algorithm",
      defaultValue: upstream.algo,
      span: 6,
      category: FormItemCategory.TEXT,
    },
    {
      id: "health_check",
      label: "Health Check",
      defaultValue: upstream.health_check,
      span: 6,
      category: FormItemCategory.TEXT,
    },
    {
      id: "connection_timeout",
      label: "Connection Timeout",
      defaultValue: upstream.connection_timeout,
      span: 6,
      category: FormItemCategory.TEXT,
    },
    {
      id: "total_connection_timeout",
      label: "Total Connection Timeout",
      defaultValue: upstream.total_connection_timeout,
      span: 6,
      category: FormItemCategory.TEXT,
    },
    {
      id: "read_timeout",
      label: "Read Timeout",
      defaultValue: upstream.read_timeout,
      span: 6,
      category: FormItemCategory.TEXT,
    },
    {
      id: "write_timeout",
      label: "Write Timeout",
      defaultValue: upstream.write_timeout,
      span: 6,
      category: FormItemCategory.TEXT,
    },
    {
      id: "idle_timeout",
      label: "Idle Timeout",
      defaultValue: upstream.idle_timeout,
      span: 6,
      category: FormItemCategory.TEXT,
    },
  ];

  const onUpsert = async (_: string, data: Record<string, unknown>) => {
    return update("upstream", name || "", data);
  };
  return (
    <FormEditor
      key={name}
      title="Modify upstream configuration"
      description="Change the upstream configuration"
      items={arr}
      onUpsert={onUpsert}
    />
  );
}