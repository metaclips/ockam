defmodule Ockam.Transport.TCP.Client do
  @moduledoc false
  use Ockam.Worker

  alias Ockam.Message
  alias Ockam.Wire

  require Logger

  # TODO: modify this for tcp
  @wire_encoder_decoder Wire.Binary.V2

  @impl true
  def address_prefix(_options), do: "TCP_C_"

  @impl true
  def setup(options, state) do
    {host, port} = Keyword.fetch!(options, :destination)

    {protocol, inet_address} =
      case host do
        string when is_binary(string) ->
          {:inet, to_charlist(string)}

        {_, _, _, _} = ipv4 ->
          {:inet, ipv4}

        {_, _, _, _, _, _, _, _} = ipv6 ->
          {:inet6, ipv6}
      end

    # TODO: connect/3 and controlling_process/2 should be in a callback.
    case :gen_tcp.connect(inet_address, port, [
           :binary,
           protocol,
           active: true,
           packet: 2,
           nodelay: true
         ]) do
      {:ok, socket} ->
        :gen_tcp.controlling_process(socket, self())
        {:ok, Map.merge(state, %{socket: socket, inet_address: inet_address, port: port})}

      {:error, reason} ->
        Logger.error("Error starting TCP client: #{inspect(reason)}")
        ## Return `normal` so the supervisor will not restart it
        ## TODO: solve this with supervision trees instead
        {:error, :normal}
    end
  end

  @impl true
  def handle_message({:tcp, socket, data}, %{socket: socket} = state) do
    with {:ok, message} <- Wire.decode(@wire_encoder_decoder, data),
         {:ok, message} <- update_return_route(message, state) do
      Ockam.Router.route(message)
    else
      {:error, %Wire.DecodeError{} = e} -> raise e
      e -> raise e
    end

    {:ok, state}
  end

  def handle_message({:tcp_closed, _}, state) do
    {:stop, :normal, state}
  end

  def handle_message({:tcp_error, _}, state) do
    {:stop, :normal, state}
  end

  ## TODO: implement Worker API
  def handle_message(%{payload: _payload} = message, state) do
    encode_and_send_over_tcp(message, state)
    {:ok, state}
  end

  defp encode_and_send_over_tcp(message, state) do
    with {:ok, message} <- remove_itself_from_onward_route(message, state),
         {:ok, encoded_message} <- Wire.encode(@wire_encoder_decoder, message),
         :ok <- send_over_tcp(encoded_message, state) do
      :ok
    end
  end

  defp remove_itself_from_onward_route(message, %{address: address}) do
    new_onward_route =
      case Message.onward_route(message) do
        [^address | rest] -> rest
        ## TODO: error message?
        other -> other
      end

    {:ok, Map.put(message, :onward_route, new_onward_route)}
  end

  defp send_over_tcp(data, %{socket: socket}) do
    :gen_tcp.send(socket, data)
  end

  defp update_return_route(message, %{address: address}) do
    return_route = Message.return_route(message)
    {:ok, Map.put(message, :return_route, [address | return_route])}
  end
end
