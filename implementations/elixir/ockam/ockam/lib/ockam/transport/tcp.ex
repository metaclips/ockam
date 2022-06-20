defmodule Ockam.Transport.TCP do
  @moduledoc """
  TCP transport
  """

  alias Ockam.Transport.TCP.Listener
  alias Ockam.Transport.TCPAddress

  alias Ockam.Message
  alias Ockam.Router
  alias Ockam.Transport.TCP.Client

  require Logger

  def child_spec(options) do
    id = id(options)

    %{
      id: id,
      start: {__MODULE__, :start, [options]}
    }
  end

  defp id(options) do
    case Keyword.fetch(options, :listen) do
      {:ok, listen} ->
        ip = Keyword.get(listen, :ip, Listener.default_ip())
        port = Keyword.get(listen, :port, Listener.default_port())

        "TCP_LISTENER_#{TCPAddress.format_host_port(ip, port)}"

      _other ->
        "TCP_TRANSPORT"
    end
  end

  @doc """
  Start a TCP transport

  ## Parameters
  - options:
      listen: t:Listener.options()
  """
  @spec start(Keyword.t()) :: :ignore | {:error, any} | {:ok, any}
  def start(options \\ []) do
    ## TODO: do we want to stop transports?
    Router.set_message_handler(TCPAddress.type(), &handle_message/1)

    case Keyword.fetch(options, :listen) do
      {:ok, listen} -> Listener.start_link(listen)
      _other -> :ignore
    end
  end

  @spec handle_message(Ockam.Message.t()) :: :ok | {:error, any()}
  defp handle_message(message) do
    case get_destination_and_onward_route(message) do
      {:ok, destination, onward_route} ->
        ## Remove tcp address from onward route
        message_to_forward =
          Map.put(create_outgoing_message(message), :onward_route, onward_route)

        ## TODO: reuse clients when using tcp address
        with {:ok, client_address} <-
               Client.create(destination: destination) do
          Ockam.Node.send(client_address, message_to_forward)
        end

      e ->
        Logger.error(
          "Cannot forward message to tcp client: #{inspect(message)} reason: #{inspect(e)}"
        )
    end
  end

  defp get_destination_and_onward_route(message) do
    {dest_address, onward_route} =
      message
      |> Message.onward_route()
      |> List.pop_at(0)

    with true <- TCPAddress.is_tcp_address(dest_address),
         {:ok, destination} <- TCPAddress.to_host_port(dest_address) do
      {:ok, destination, onward_route}
    else
      false ->
        {:error, {:invalid_address_type, dest_address}}

      {:error, error} ->
        {:error, error}
    end
  end

  defp create_outgoing_message(message) do
    %{
      onward_route: Message.onward_route(message),
      payload: Message.payload(message),
      return_route: Message.return_route(message)
    }
  end
end
